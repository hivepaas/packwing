package com.hivepaas.packwing

import android.os.Handler
import android.os.Looper
import uniffi.packwing_mobile.SshQuestions
import uniffi.packwing_mobile.TypedSecret
import java.util.concurrent.CountDownLatch

/** What SFTP asks as it connects. */
sealed interface SshQuestion {
    /** A server packwing has not connected to before: trust it? */
    data class HostKey(val host: String, val port: UShort, val algorithm: String, val fingerprint: String) : SshQuestion

    /** The password of `target` (`user@host`); `retry` when the last was refused. */
    data class Password(val target: String, val retry: Boolean) : SshQuestion

    /** The passphrase of the key `key`; `retry` when the last did not open it. */
    data class Passphrase(val key: String, val retry: Boolean) : SshQuestion

    /** What the server asks besides, such as a one-time code. */
    data class Code(val name: String, val instructions: String, val text: String, val echo: Boolean) : SshQuestion
}

/** What the user said: yes or no to a server, a secret typed, a code; nothing for no. */
sealed interface SshAnswer {
    data object None : SshAnswer
    data class Trust(val yes: Boolean) : SshAnswer
    data class Secret(val typed: TypedSecret?) : SshAnswer
    data class Code(val code: String?) : SshAnswer
}

/** A question shown, and where its answer goes. */
class Asked(val question: SshQuestion, private val answered: (SshAnswer) -> Unit) {
    fun answer(said: SshAnswer) = answered(said)
}

/**
 * SFTP's questions, for the core: it asks on the worker thread a storage's
 * call runs on, and waits. The question is shown over whatever shows (a
 * sheet, a job's screen), and its answer goes back to that thread.
 */
class SshAsker(private val model: AppModel) : SshQuestions {
    private val main = Handler(Looper.getMainLooper())

    override fun hostKey(host: String, port: UShort, algorithm: String, fingerprint: String): Boolean =
        (ask(SshQuestion.HostKey(host, port, algorithm, fingerprint)) as? SshAnswer.Trust)?.yes ?: false

    override fun password(target: String, retry: Boolean): TypedSecret? =
        (ask(SshQuestion.Password(target, retry)) as? SshAnswer.Secret)?.typed

    override fun passphrase(key: String, retry: Boolean): TypedSecret? =
        (ask(SshQuestion.Passphrase(key, retry)) as? SshAnswer.Secret)?.typed

    override fun code(name: String, instructions: String, text: String, echo: Boolean): String? =
        (ask(SshQuestion.Code(name, instructions, text, echo)) as? SshAnswer.Code)?.code

    /** Shows `question`, and waits for its answer. Never on the main thread, which the answer comes from. */
    private fun ask(question: SshQuestion): SshAnswer {
        if (Looper.myLooper() == Looper.getMainLooper()) return SshAnswer.None
        val done = CountDownLatch(1)
        var said: SshAnswer = SshAnswer.None
        val asked = Asked(question) { answer ->
            said = answer
            done.countDown()
        }
        // In turn: one asked while another is shown waits for it.
        main.post { model.questions.add(asked) }
        done.await()
        return said
    }
}
