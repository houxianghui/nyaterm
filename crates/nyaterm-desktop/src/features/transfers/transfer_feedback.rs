use crate::features::NyaTermApp;
use crate::features::android::labels::{file_error_message, log_file_error};
use crate::features::transfers::FileTransferSession;
use gpui::Context;
use nyaterm_files::file_browser::{FileBrowserBackendKind, FileTransferEntryError};
use nyaterm_ui::notification::{NyaNotificationAction, NyaNotificationKind};

impl NyaTermApp {
    pub(in crate::features) fn browser_transfer_session(
        &mut self,
    ) -> anyhow::Result<FileTransferSession> {
        let id = self
            .session
            .active_id_owned()
            .ok_or_else(|| anyhow::Error::new(FileTransferEntryError::Unavailable))?;
        anyhow::ensure!(
            !self.session.is_disconnected(&id),
            FileTransferEntryError::Unavailable
        );
        let service = self.file_browser_service_for_session(&id)?;
        anyhow::ensure!(
            service.capabilities().ordinary_transfer,
            FileTransferEntryError::UnsupportedBackend
        );
        if let nyaterm_files::file_browser::FileBrowserService::Adb(service) = &service {
            anyhow::ensure!(
                !service.control.is_cancelled(),
                FileTransferEntryError::Unavailable
            );
        }
        Ok(FileTransferSession {
            session_id: Some(id),
            service,
        })
    }

    pub(in crate::features) fn report_transfer_entry_error(
        &mut self,
        operation: &'static str,
        backend: FileBrowserBackendKind,
        error: &anyhow::Error,
        cx: &mut Context<Self>,
    ) {
        let id = self.transfer.next_transfer_job_id("file-request");
        log_file_error(operation, backend, &id, error);
        let message = file_error_message(error);
        self.shell.set_status(message.clone());
        self.transfer.set_browser_status(message.clone());
        // The toast only carries a short, user-facing message; "View Logs"
        // keeps the full error available through the runtime log directory.
        let app = cx.entity().downgrade();
        let action = NyaNotificationAction::new(
            rust_i18n::t!("menu.viewLogs").to_string(),
            move |_, _, cx| {
                if let Some(app) = app.upgrade() {
                    app.update(cx, |app, cx| app.reveal_log_dir(cx));
                }
            },
        );
        self.notify_operation_with_action(
            "file-transfer-error",
            NyaNotificationKind::Error,
            message,
            action,
            cx,
        );
        cx.notify();
    }

    pub(in crate::features) fn report_browser_transfer_error(
        &mut self,
        operation: &'static str,
        error: &anyhow::Error,
        cx: &mut Context<Self>,
    ) {
        let backend = self
            .session
            .active_file_browser_backend()
            .unwrap_or(FileBrowserBackendKind::Local);
        self.report_transfer_entry_error(operation, backend, error, cx);
    }
}
