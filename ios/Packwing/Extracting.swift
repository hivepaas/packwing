import Foundation
import Photos
import SwiftUI
import UIKit

/// Where extracted files go.
enum ExtractTarget: Equatable {
    /// A new folder, named as the archive, in this one.
    case files(URL)
    /// The pictures and videos go to Photos; the rest stays in the archive.
    case photos
}

extension AppModel {
    // MARK: Previewing

    /// Shows the file at `path` of `opened` in Quick Look, from a copy of it.
    func preview(_ opened: OpenedArchive, _ path: [String]) {
        guard reading == nil else { return }
        let job = Job.quiet()
        readJob = job
        reading = path.last
        Task {
            let result = await Task.detached(priority: .userInitiated) {
                Result { try opened.archive.openCopy(entry: path, job: job) }
            }.value
            guard readJob === job else { return }
            reading = nil
            readJob = nil
            switch result {
            case .success(let copy):
                previewing = URL(filePath: copy)
            case .failure(let error):
                if case .Cancelled = error as? MobileError { return }
                failure = Self.said(error)
            }
        }
    }

    func stopReading() {
        readJob?.cancel()
        readJob = nil
        reading = nil
    }

    // MARK: Where files go

    /// packwing's own folder, which Files shows as On My iPhone › packwing.
    static var ownFolder: URL { URL.documentsDirectory }

    /// Where Extract puts its folders: the one chosen last, else packwing's own.
    var extractFolder: URL { keptFolder("extractTo") }

    /// The folder kept under `key`, else packwing's own.
    func keptFolder(_ key: String) -> URL {
        guard let data = UserDefaults.standard.data(forKey: key) else { return Self.ownFolder }
        var stale = false
        return (try? URL(resolvingBookmarkData: data, bookmarkDataIsStale: &stale)) ?? Self.ownFolder
    }

    /// `folder` as the person knows it.
    func shown(_ folder: URL) -> String {
        folder.standardizedFileURL == Self.ownFolder.standardizedFileURL
            ? tr("On My iPhone › packwing")
            : folder.lastPathComponent
    }

    /// Keeps `folder`, chosen in Files, for the next extractions too.
    func extractTo(_ folder: URL) { keep(folder, as: "extractTo") }

    /// Keeps `folder`, chosen in Files, under `key`.
    func keep(_ folder: URL, as key: String) {
        let scoped = folder.startAccessingSecurityScopedResource()
        defer { if scoped { folder.stopAccessingSecurityScopedResource() } }
        guard let data = try? folder.bookmarkData() else { return }
        UserDefaults.standard.set(data, forKey: key)
        foldersChanged += 1
    }

    // MARK: Extracting

    /// Extracts `picked` of `opened` (all of it if none) to `target`, showing
    /// how far it is.
    func extract(_ opened: OpenedArchive, picked: [[String]], to target: ExtractTarget) {
        if let job, job.state == .running {
            showingJob = true
            return
        }
        let running = RunningJob(
            title: target == .photos ? tr("Saving to Photos") : tr("Extracting"),
            name: opened.info.name)
        job = running
        showingJob = true
        running.continueInBackground(subtitle: opened.info.name)
        Task {
            let end: RunningJob.State
            switch target {
            case .files(let parent):
                end = await extractToFiles(opened, picked, parent, running)
            case .photos:
                end = await saveToPhotos(opened, picked, running)
            }
            running.finish(end)
        }
    }

    private func extractToFiles(
        _ opened: OpenedArchive, _ picked: [[String]], _ parent: URL, _ running: RunningJob
    ) async -> RunningJob.State {
        let scoped = parent.startAccessingSecurityScopedResource()
        defer { if scoped { parent.stopAccessingSecurityScopedResource() } }
        let folder = Self.freeFolder(in: parent, named: stem(name: opened.info.name))
        let job = running.job
        let result = await Task.detached(priority: .userInitiated) {
            Result {
                try opened.archive.extract(
                    destination: folder.path(percentEncoded: false), picked: picked,
                    how: ExtractHow(overwrite: false), job: job)
            }
        }.value
        switch result {
        case .success(let done):
            running.folder = folder
            return .done(detail: trN(Int(done.files), "{n} file in {folder}", "{n} files in {folder}",
                                     ["folder": folder.lastPathComponent]))
        case .failure(let error):
            return Self.ended(by: error)
        }
    }

    private func saveToPhotos(
        _ opened: OpenedArchive, _ picked: [[String]], _ running: RunningJob
    ) async -> RunningJob.State {
        let status = await PHPhotoLibrary.requestAuthorization(for: .addOnly)
        guard status == .authorized || status == .limited else {
            return .failed(message: tr("packwing may not add to Photos. Settings › Privacy › Photos lets it."))
        }
        let temp = URL.temporaryDirectory.appending(path: "photos-\(UUID().uuidString)")
        defer { try? FileManager.default.removeItem(at: temp) }
        let job = running.job
        let result = await Task.detached(priority: .userInitiated) {
            Result {
                try opened.archive.extract(
                    destination: temp.path(percentEncoded: false), picked: picked,
                    how: ExtractHow(overwrite: false), job: job)
            }
        }.value
        if case .failure(let error) = result { return Self.ended(by: error) }
        let media = Self.media(in: temp)
        guard !media.isEmpty else {
            return .failed(message: tr("There are no pictures or videos in it."))
        }
        do {
            try await PHPhotoLibrary.shared().performChanges {
                for (url, video) in media {
                    PHAssetCreationRequest.forAsset()
                        .addResource(with: video ? .video : .photo, fileURL: url, options: nil)
                }
            }
        } catch {
            return .failed(message: error.localizedDescription)
        }
        return .done(detail: trN(media.count, "{n} picture or video added to Photos",
                                 "{n} pictures and videos added to Photos"))
    }

    /// The pictures and videos under `folder`, and which are videos.
    private static func media(in folder: URL) -> [(URL, Bool)] {
        let files = FileManager.default.enumerator(at: folder, includingPropertiesForKeys: [.isRegularFileKey])
        var found: [(URL, Bool)] = []
        while let url = files?.nextObject() as? URL {
            switch FileKind.of(name: url.lastPathComponent) {
            case .image where url.pathExtension.lowercased() != "svg": found.append((url, false))
            case .video: found.append((url, true))
            default: break
            }
        }
        return found
    }

    /// `name` in `parent`, or `name 2`, `name 3`… if that is taken.
    static func freeFolder(in parent: URL, named name: String) -> URL {
        let base = name.isEmpty ? tr("Extracted") : name
        var n = 1
        while true {
            let candidate = parent.appending(path: n == 1 ? base : "\(base) \(n)", directoryHint: .isDirectory)
            if !FileManager.default.fileExists(atPath: candidate.path(percentEncoded: false)) { return candidate }
            n += 1
        }
    }

    static func ended(by error: Error) -> RunningJob.State {
        if case .Cancelled = error as? MobileError { return .cancelled }
        return .failed(message: said(error))
    }

    /// What `error` says, in the person's words.
    static func said(_ error: Error) -> String {
        switch error as? MobileError {
        case .NotAnArchive(let m), .Locked(let m, _), .WrongPassword(let m), .Cancelled(let m), .Failed(let m): m
        case nil: error.localizedDescription
        }
    }

    /// Opens Files at `folder`.
    func showInFiles(_ folder: URL) {
        var parts = URLComponents()
        parts.scheme = "shareddocuments"
        parts.path = folder.path(percentEncoded: false)
        if let url = parts.url { UIApplication.shared.open(url) }
    }
}
