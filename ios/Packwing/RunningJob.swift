import BackgroundTasks
import Foundation
import Observation

/// A long job (extracting, saving to Photos), as its screen shows it: how far
/// it is, and how it ended. It runs whether or not iOS lets it go on in the
/// background; when it does (BGContinuedProcessingTask), iOS shows its
/// progress while packwing is left, and a stop there cancels it.
@MainActor
@Observable
final class RunningJob: Identifiable {
    enum State: Equatable {
        case running
        /// `detail` says what was done.
        case done(detail: String)
        case failed(message: String)
        case cancelled
    }

    let id = UUID()
    /// What it does, as its screen's title says it: "Extracting".
    var title: String
    /// The archive it is of.
    let name: String
    var progress = JobProgress(entries: 0, totalEntries: 0, bytes: 0, totalBytes: 0)
    var state: State = .running
    /// The folder the files went to, to show in Files.
    var folder: URL?
    let started = Date()

    @ObservationIgnored let job: Job
    @ObservationIgnored private let listener: Listener
    @ObservationIgnored private var task: BGContinuedProcessingTask?

    init(title: String, name: String) {
        self.title = title
        self.name = name
        let listener = Listener()
        self.listener = listener
        job = Job(listener: listener)
        listener.owner = self
    }

    /// Done, as a share of the bytes to write: 0 to 1.
    var fraction: Double {
        progress.totalBytes == 0 ? 0 : min(1, Double(progress.bytes) / Double(progress.totalBytes))
    }

    /// About how long is left, once a little is done.
    var timeLeft: TimeInterval? {
        let done = fraction
        let elapsed = Date().timeIntervalSince(started)
        guard done > 0.02, elapsed > 1 else { return nil }
        return elapsed * (1 - done) / done
    }

    func cancel() {
        job.cancel()
    }

    fileprivate func told(_ p: JobProgress) {
        guard state == .running else { return }
        progress = p
        if let task {
            task.progress.totalUnitCount = Int64(clamping: max(p.totalBytes, 1))
            task.progress.completedUnitCount = Int64(clamping: p.bytes)
        }
    }

    func finish(_ end: State) {
        state = end
        if case .done = end, progress.totalBytes > 0 {
            progress.bytes = progress.totalBytes
            progress.entries = progress.totalEntries
        }
        task?.setTaskCompleted(success: { if case .done = end { true } else { false } }())
        task = nil
    }

    // MARK: Going on in the background

    /// Asks iOS to let it go on once packwing is left, showing `subtitle`.
    /// Nothing changes if iOS says no: it goes on while packwing is open.
    func continueInBackground(subtitle: String) {
        let identifier = "com.hivepaas.packwing.job.\(id.uuidString)"
        let registered = BGTaskScheduler.shared.register(forTaskWithIdentifier: identifier, using: nil) { [weak self] task in
            guard let task = task as? BGContinuedProcessingTask else {
                task.setTaskCompleted(success: false)
                return
            }
            Task { @MainActor in
                guard let self else {
                    task.setTaskCompleted(success: false)
                    return
                }
                self.attach(task)
            }
        }
        guard registered else { return }
        let request = BGContinuedProcessingTaskRequest(identifier: identifier, title: title, subtitle: subtitle)
        // Now or not at all: a job waiting behind others is no use.
        request.strategy = .fail
        try? BGTaskScheduler.shared.submit(request)
    }

    private func attach(_ task: BGContinuedProcessingTask) {
        guard state == .running else {
            task.setTaskCompleted(success: true)
            return
        }
        self.task = task
        let job = job
        task.expirationHandler = { job.cancel() }
        told(progress)
    }
}

/// Told by the core, off the main thread.
private final class Listener: JobListener, @unchecked Sendable {
    weak var owner: RunningJob?

    func progress(progress: JobProgress) {
        Task { @MainActor [weak self] in
            self?.owner?.told(progress)
        }
    }
}
