"""Attach the tested ZIP to a draft GitHub release for its verified tag."""

import argparse
import json
from pathlib import Path
import subprocess


RELEASE_QUERY = """query($owner: String!, $name: String!, $tag: String!) {
  repository(owner: $owner, name: $name) { release(tagName: $tag) { isDraft } }
}"""


def prepare(repository, tag, archive):
    if not archive.is_file():
        raise FileNotFoundError(archive)
    owner, name = repository.split("/")
    lookup = subprocess.run(
        ["gh", "api", "graphql", "-f", f"query={RELEASE_QUERY}", "-f", f"owner={owner}",
         "-f", f"name={name}", "-f", f"tag={tag}"],
        check=True, stdout=subprocess.PIPE, text=True,
    )
    result = json.loads(lookup.stdout)
    repo = (result.get("data") or {}).get("repository")
    if result.get("errors") or not isinstance(repo, dict) or "release" not in repo:
        raise RuntimeError(f"Could not establish the release state for {repository} {tag}.")
    release = repo["release"]
    if release is None:
        version = tag.removeprefix("ethos-model-editor-v")
        subprocess.run(["gh", "release", "create", tag, str(archive), "--repo", repository,
                        "--verify-tag", "--draft", "--title", f"ETHOS Model Editor {version}",
                        "--generate-notes"], check=True)
    elif isinstance(release, dict) and release.get("isDraft") is True:
        subprocess.run(["gh", "release", "upload", tag, str(archive), "--repo", repository,
                        "--clobber"], check=True)
    elif isinstance(release, dict) and release.get("isDraft") is False:
        print(f"::notice::Release {tag} is already published. Its assets were preserved.")
    else:
        raise RuntimeError(f"Unrecognized release state for {repository} {tag}.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("repository")
    parser.add_argument("tag")
    parser.add_argument("archive", type=Path)
    args = parser.parse_args()
    prepare(args.repository, args.tag, args.archive)
