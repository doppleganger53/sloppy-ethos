import json
from pathlib import Path
import unittest
from unittest.mock import patch

from prepare_release import prepare


class PrepareReleaseTests(unittest.TestCase):
    @patch("prepare_release.subprocess.run")
    def test_missing_repository_fails_without_mutation(self, run):
        run.return_value.stdout = json.dumps({"data": {"repository": None}})
        with self.assertRaisesRegex(RuntimeError, "establish the release state"):
            prepare("example/editor", "ethos-model-editor-v1", Path(__file__))
        self.assertEqual(run.call_count, 1)

    @patch("prepare_release.subprocess.run")
    def test_refreshed_drafts_and_published_releases(self, run):
        for release, mutation in [(None, "create"), ({"isDraft": True}, "upload"),
                                   ({"isDraft": False}, None), ({"isDraft": None}, "error")]:
            with self.subTest(release=release):
                run.reset_mock()
                run.return_value.stdout = json.dumps({"data": {"repository": {"release": release}}})
                if mutation == "error":
                    with self.assertRaises(RuntimeError):
                        prepare("example/editor", "ethos-model-editor-v1", Path(__file__))
                else:
                    prepare("example/editor", "ethos-model-editor-v1", Path(__file__))
                commands = [call.args[0] for call in run.call_args_list]
                self.assertEqual(commands[0][:3], ["gh", "api", "graphql"])
                if mutation in {"create", "upload"}:
                    self.assertEqual(len(commands), 2)
                    self.assertEqual(commands[1][:3], ["gh", "release", mutation])
                    self.assertIn("--draft" if mutation == "create" else "--clobber", commands[1])
                else:
                    self.assertEqual(len(commands), 1)
