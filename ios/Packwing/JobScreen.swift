import SwiftUI

/// A long job, full screen: how far it is, then how it ended.
struct JobScreen: View {
    @Environment(AppModel.self) private var model
    let job: RunningJob
    @AppStorage("showInFiles") private var showInFiles = false

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Button {
                    model.showingJob = false
                } label: {
                    Image(systemName: "chevron.down")
                        .font(.system(size: 18, weight: .semibold))
                        .frame(width: 44, height: 44)
                }
                .buttonStyle(.glass)
                .accessibilityLabel(job.state == .running ? tr("Hide, and keep going") : tr("Close"))
                Spacer()
            }
            .padding(.horizontal, 16)
            .padding(.top, 8)

            ScrollView {
                VStack(spacing: 24) {
                    Ring(job: job)
                    VStack(spacing: 4) {
                        Text(heading)
                            .font(.nunito(26, .bold, relativeTo: .title))
                        Text(job.name)
                            .font(.nunito(15, relativeTo: .subheadline))
                            .foregroundStyle(Tonal.muted)
                            .lineLimit(2)
                            .multilineTextAlignment(.center)
                    }
                    if job.progress.totalEntries > 0 {
                        HStack(spacing: 8) {
                            Stat(value: "\(job.progress.entries) / \(job.progress.totalEntries)", label: tr("files and folders"))
                            Stat(value: sizeText(job.progress.bytes),
                                 label: tr("of {size}", ["size": sizeText(job.progress.totalBytes)]))
                        }
                    }
                    note
                }
                .padding(.horizontal, 24)
                .padding(.top, 16)
            }

            footer
                .padding(.horizontal, 20)
                .padding(.bottom, 12)
        }
        .background(Tonal.page)
        .onChange(of: job.state) { _, now in
            // Extracted files are shown as asked; a new archive is shared, or not.
            if case .done = now, showInFiles, job.made == nil, let folder = job.folder {
                model.showInFiles(folder)
            }
        }
    }

    private var heading: String {
        switch job.state {
        case .running: job.title
        case .done: tr("Done")
        case .failed: tr("It didn’t finish")
        case .cancelled: tr("Cancelled")
        }
    }

    @ViewBuilder private var note: some View {
        switch job.state {
        case .running:
            Label(tr("You can leave packwing. iOS keeps it going and shows how far it is."),
                  systemImage: "iphone")
                .noteStyle(Tonal.tonal, Tonal.onTonal)
        case .done(let detail):
            Label(detail, systemImage: "checkmark.circle")
                .noteStyle(Tonal.tonal, Tonal.onTonal)
        case .failed(let message):
            Label(message, systemImage: "exclamationmark.triangle")
                .noteStyle(Tonal.badBg, Tonal.badText)
        case .cancelled:
            Label(tr("Nothing was left behind."), systemImage: "xmark.circle")
                .noteStyle(Tonal.field, Tonal.text)
        }
    }

    private var done: some View {
        Button {
            model.showingJob = false
        } label: {
            Text(tr("Done"))
                .font(.nunito(17, .semibold, relativeTo: .headline))
                .frame(maxWidth: .infinity, minHeight: 52)
        }
        .buttonStyle(.glass)
    }

    @ViewBuilder private var footer: some View {
        switch job.state {
        case .running:
            Button(role: .cancel) { job.cancel() } label: {
                Text(tr("Cancel"))
                    .font(.nunito(17, .semibold, relativeTo: .headline))
                    .frame(maxWidth: .infinity, minHeight: 52)
            }
            .buttonStyle(.glass)
        default:
            VStack(spacing: 10) {
                if case .done = job.state, let made = job.made {
                    ShareLink(item: made) {
                        Label(tr("Share"), systemImage: "square.and.arrow.up")
                            .font(.nunito(17, .bold, relativeTo: .headline))
                            .frame(maxWidth: .infinity, minHeight: 52)
                    }
                    .buttonStyle(.glassProminent)
                }
                if case .done = job.state, let folder = job.folder {
                    if job.made == nil {
                        Button { model.showInFiles(folder) } label: {
                            Text(tr("Show in Files"))
                                .font(.nunito(17, .bold, relativeTo: .headline))
                                .frame(maxWidth: .infinity, minHeight: 52)
                        }
                        .buttonStyle(.glassProminent)
                        done
                    } else {
                        // Under Share, side by side: the note above stays in sight.
                        HStack(spacing: 10) {
                            Button { model.showInFiles(folder) } label: {
                                Text(tr("Show in Files"))
                                    .font(.nunito(17, .semibold, relativeTo: .headline))
                                    .lineLimit(1)
                                    .minimumScaleFactor(0.8)
                                    .frame(maxWidth: .infinity, minHeight: 52)
                            }
                            .buttonStyle(.glass)
                            done
                        }
                    }
                } else {
                    done
                }
            }
        }
    }
}

/// How far, as a ring with its share in the middle.
private struct Ring: View {
    let job: RunningJob

    var body: some View {
        ZStack {
            Circle().stroke(Tonal.field, lineWidth: 18)
            Circle()
                .trim(from: 0, to: job.fraction)
                .stroke(Tonal.accent, style: StrokeStyle(lineWidth: 18, lineCap: .round))
                .rotationEffect(.degrees(-90))
                .animation(.linear(duration: 0.2), value: job.fraction)
            VStack(spacing: 2) {
                Text(job.fraction, format: .percent.precision(.fractionLength(0)))
                    .font(.nunito(48, .bold, relativeTo: .largeTitle))
                    .foregroundStyle(Tonal.onTonal)
                    .monospacedDigit()
                if job.state == .running, let left = job.timeLeft {
                    Text(tr("about {time} left",
                            ["time": Duration.seconds(left).formatted(.units(allowed: [.hours, .minutes, .seconds], width: .narrow, maximumUnitCount: 1).locale(appLocale))]))
                        .font(.nunito(14, relativeTo: .subheadline))
                        .foregroundStyle(Tonal.muted)
                }
            }
        }
        .frame(width: 220, height: 220)
        .accessibilityElement(children: .combine)
    }
}

private struct Stat: View {
    let value: String
    let label: String

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(value).font(.nunito(20, .bold, relativeTo: .title3)).monospacedDigit()
            Text(label).font(.nunito(13, relativeTo: .footnote)).foregroundStyle(Tonal.muted)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(14)
        .background(Tonal.surface, in: .rect(cornerRadius: 22))
    }
}

private extension View {
    func noteStyle(_ ground: Color, _ ink: Color) -> some View {
        font(.nunito(14, relativeTo: .subheadline))
            .foregroundStyle(ink)
            .padding(.horizontal, 16)
            .padding(.vertical, 14)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(ground, in: .rect(cornerRadius: 22))
    }
}

/// A job going on while its screen is hidden: tapped, it shows again.
struct JobBanner: View {
    @Environment(AppModel.self) private var model
    let job: RunningJob

    var body: some View {
        Button {
            model.showingJob = true
        } label: {
            HStack(spacing: 12) {
                ProgressView(value: job.fraction)
                    .progressViewStyle(.circular)
                VStack(alignment: .leading, spacing: 1) {
                    Text(job.title)
                        .font(.nunito(13, .medium, relativeTo: .caption))
                        .foregroundStyle(Tonal.muted)
                    Text(job.name)
                        .font(.nunito(15, .semibold, relativeTo: .subheadline))
                        .lineLimit(1)
                }
                Spacer(minLength: 8)
                Text(job.fraction, format: .percent.precision(.fractionLength(0)))
                    .font(.nunito(15, .bold, relativeTo: .subheadline))
                    .monospacedDigit()
            }
            .foregroundStyle(Tonal.text)
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
        }
        .buttonStyle(.plain)
        .glassEffect(.regular.interactive(), in: .capsule)
        .padding(.horizontal, 20)
    }
}
