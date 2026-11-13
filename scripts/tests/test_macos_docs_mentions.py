import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class DocsTest(unittest.TestCase):
    def test_migration_doc_documents_exit_dialog_and_tab_keys(self):
        text = (ROOT / "docs/macos-visual-migration.md").read_text(encoding="utf-8")
        heading = "### 3.3 确认对话框与终端页签"
        self.assertIn(heading, text)
        section = text.split(heading, 1)[1].split("\n## ", 1)[0]
        for key in (
            "Enter",
            "Escape",
            "Ctrl+Tab",
            "Ctrl+Shift+Tab",
            "Ctrl+Shift+W",
            "Cmd+Tab",
            "Cmd+Shift+Tab",
            "Cmd+W",
            "Cmd+Shift+W",
        ):
            with self.subTest(key=key):
                self.assertIn(key, section)
