//! The questions a connection over SSH asks, from a storage's worker thread: whether
//! to trust a server, and the secrets it needs.

use std::thread::{self, JoinHandle};

use packwing_remote::{HostKeyQuestion, PasswordQuestion, Questions, Typed};

use super::{Ui, scenario};
use crate::ssh::Asked;

fn host() -> HostKeyQuestion {
    HostKeyQuestion {
        host: "files.example.com".into(),
        port: 2222,
        algorithm: "ED25519".into(),
        fingerprint: "SHA256:k3HQhz0oGq4nV8YQqNqWm2U7uXo9yH2bL1cJpR4tE5s".into(),
    }
}

/// What the connection on `asking` got, once the window answered it.
fn answered<T>(ui: &mut Ui, asking: JoinHandle<T>) -> T {
    ui.wait_until("the answer", |_| asking.is_finished());
    asking.join().expect("answered")
}

/// Opens the dialog that adds a storage, as Storages' button does.
fn add_storage_open(ui: &mut Ui) {
    ui.ws(|ws, window, cx| ws.open_add_storage(window, cx));
    ui.wait_for("kind-0");
}

scenario!(
    /// A server packwing has not met is trusted only when the user says so; Escape
    /// says no.
    a_server_is_trusted_only_when_the_user_says_so,
    |ui| {
        let asker = ui.ws(|ws, _, _| ws.asker());
        let asking = asker.clone();
        let asked = thread::spawn(move || asking.host_key(host()));
        ui.wait_for("ssh-fingerprint");
        assert_eq!(
            ui.ws(|ws, _, cx| ws.ssh_asked(cx)),
            Some(Asked::HostKey(host()))
        );
        ui.click("ssh-yes");
        assert!(answered(ui, asked));
        ui.wait_until("no question", |ui| !ui.shows("ssh-question"));

        let asking = asker.clone();
        let asked = thread::spawn(move || asking.host_key(host()));
        ui.wait_for("ssh-question");
        ui.keys("escape");
        assert!(!answered(ui, asked));
        ui.wait_until("no question", |ui| !ui.shows("ssh-question"));
    }
);

scenario!(
    /// A password typed goes back to the connection, kept only when the user asks.
    a_password_is_kept_only_when_asked,
    |ui| {
        let asker = ui.ws(|ws, _, _| ws.asker());
        let question = PasswordQuestion {
            target: "me@files.example.com".into(),
            retry: false,
        };
        let (asking, q) = (asker.clone(), question.clone());
        let asked = thread::spawn(move || asking.password(q));
        ui.wait_for("ssh-question");
        // Nothing typed, nothing sent.
        ui.click("ssh-yes");
        assert!(ui.shows("ssh-question"));
        ui.type_in("Password", "hunter2");
        ui.click("ssh-keep");
        ui.click("ssh-yes");
        let typed = Typed {
            secret: "hunter2".into(),
            keep: true,
        };
        assert_eq!(answered(ui, asked), Some(typed));

        let asking = asker.clone();
        let asked = thread::spawn(move || asking.password(question));
        ui.wait_for("ssh-question");
        ui.click("ssh-no");
        assert_eq!(answered(ui, asked), None);
    }
);

scenario!(
    /// A question comes over the dialog open, as a storage's test of its connection
    /// asks one, and leaves it there once answered; questions are asked one at a time.
    a_question_comes_over_a_dialog_and_leaves_it_open,
    |ui| {
        add_storage_open(ui);
        let asker = ui.ws(|ws, _, _| ws.asker());
        let (one, two) = (asker.clone(), asker.clone());
        let first = thread::spawn(move || one.host_key(host()));
        let second = thread::spawn(move || two.host_key(host()));
        ui.wait_for("ssh-question");
        ui.click("ssh-yes");
        ui.wait_for("ssh-question");
        ui.click("ssh-no");
        let answers = [
            answered(ui, first),
            answered(ui, second),
        ];
        assert_eq!(answers.iter().filter(|a| **a).count(), 1);
        ui.wait_until("no question", |ui| !ui.shows("ssh-question"));
        assert!(ui.dialog_open());
        assert!(ui.shows("kind-0"));
    }
);
