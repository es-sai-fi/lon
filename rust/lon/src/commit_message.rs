use crate::sources::{RevisionUpdate, UpdateSummary};

use std::fmt::{self, Write};

pub struct CommitMessage {
    updates: Vec<(String, UpdateSummary)>,
}

impl CommitMessage {
    pub fn new() -> Self {
        Self { updates: vec![] }
    }

    pub fn add_summary(&mut self, name: &str, summary: UpdateSummary) {
        self.updates.push((name.into(), summary));
    }

    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
    }

    /// Construct the body of the commit message.
    pub fn body(&self) -> std::result::Result<String, fmt::Error> {
        let mut commit_message = String::new();

        if self.updates.len() == 1 {
            let summary = &self.updates[0].1;

            writeln!(&mut commit_message)?;
            match summary {
                UpdateSummary::Rev(summary) => {
                    writeln!(&mut commit_message, "  {}", summary.old_revision)?;
                    writeln!(&mut commit_message, "→ {}", summary.new_revision)?;

                    if let Some(rev_list_overview) = Self::rev_list_overview(summary, 0) {
                        writeln!(&mut commit_message)?;
                        writeln!(&mut commit_message, "{rev_list_overview}")?;
                    }
                }
                UpdateSummary::Url(summary) => {
                    writeln!(&mut commit_message, "  {}", summary.old_url)?;
                    writeln!(&mut commit_message, "→ {}", summary.new_url)?;
                }
            }
        } else {
            for (name, summary) in &self.updates {
                writeln!(&mut commit_message)?;
                writeln!(&mut commit_message, "• {name}:")?;
                match summary {
                    UpdateSummary::Rev(summary) => {
                        writeln!(&mut commit_message, "    {}", summary.old_revision)?;
                        writeln!(&mut commit_message, "  → {}", summary.new_revision)?;

                        if let Some(rev_list_overview) = Self::rev_list_overview(summary, 2) {
                            writeln!(&mut commit_message)?;
                            writeln!(&mut commit_message, "{rev_list_overview}")?;
                        }
                    }
                    UpdateSummary::Url(summary) => {
                        writeln!(&mut commit_message, "    {}", summary.old_url)?;
                        writeln!(&mut commit_message, "  → {}", summary.new_url)?;
                    }
                }
            }
        }
        Ok(commit_message)
    }

    /// Construct the overview of the rev list from a summary.
    ///
    /// Adds whitespace according to the ident argument.
    fn rev_list_overview(summary: &RevisionUpdate, indent: usize) -> Option<String> {
        summary.rev_list.as_ref().map(|revs| {
            let prefix = " ".repeat(indent);
            let revs = revs.revs();

            std::iter::once(format!("{prefix}Last {} commits:", revs.len()))
                .chain(revs.iter().map(|commit| {
                    format!(
                        "\n{prefix}  {} {}",
                        commit.revision.short(),
                        commit.message_summary(),
                    )
                }))
                .collect::<Vec<String>>()
                .concat()
        })
    }
}

impl fmt::Display for CommitMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut commit_message = String::new();

        if self.updates.len() == 1 {
            let name = &self.updates[0].0;
            writeln!(&mut commit_message, "lon: update {name}")?;
        } else {
            writeln!(&mut commit_message, "lon: update")?;
        }
        write!(&mut commit_message, "{}", self.body()?)?;
        write!(f, "{commit_message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use expect_test::expect;
    use indoc::indoc;

    use crate::{
        git::{Commit, RevList, Revision},
        sources::RevisionUpdate,
    };

    fn summary_1() -> UpdateSummary {
        UpdateSummary::from_revs(
            Revision::new("043344a1c19619435e2b79cd42de6592308af0aa"),
            Revision::new("21386f9d14831b594048e1e4340ac7a300e312d6"),
        )
    }

    fn summary_2() -> UpdateSummary {
        UpdateSummary::from_revs(
            Revision::new("ad3bc97747c651e23fbc12c70a5849d3d8e9fdf4"),
            Revision::new("75962bcd89dcccc9fe125c9ab46377d6cd1ddb00"),
        )
    }

    /// Summary with a rev list from git commandline
    fn summary_rev_list_1() -> UpdateSummary {
        let mut summary = RevisionUpdate::new(
            Revision::new("043344a1c19619435e2b79cd42de6592308af0aa"),
            Revision::new("21386f9d14831b594048e1e4340ac7a300e312d6"),
        );
        let rev_list_git_output = indoc! {"
            1ba800e readme: reorganize
            26244f0 readme: add section about bot
            c67d352 changelog: add entry about bot
            5de6d54 bot: init
        "};
        let rev_list = RevList::from_git_output(rev_list_git_output);
        summary.add_rev_list(rev_list);
        UpdateSummary::Rev(summary)
    }

    /// Summary with a rev list from GitHub
    fn summary_rev_list_2() -> UpdateSummary {
        let mut summary = RevisionUpdate::new(
            Revision::new("6c1da4c913f0edf2835c3cc47c3889c36c05e6ca"),
            Revision::new("629f1e13eb7d09738538ba1b3c2ce35d9c1bef3e"),
        );
        // Long message to test that it only shows the summary line
        let msg = indoc! {"
            emacs: remove native-comp-compiler-options-28.patch

            Since we do not deal with older Emacsen.

            P.S.: We can't delete native-comp-compiler-options.patch yet, because Emacs
            Macport is stuck in Emacs 29.
        "};
        let rev_list = RevList::from_commits(vec![
            Commit::from_str("1ba800e", msg),
            Commit::from_str("26244f0", "readme: add section about bot"),
            Commit::from_str("6232894", ".gitignore: ignore .env"),
        ]);
        summary.add_rev_list(rev_list);
        UpdateSummary::Rev(summary)
    }

    #[test]
    fn commit_message_single_update() {
        let mut commit_message = CommitMessage::new();
        commit_message.add_summary("fake_1", summary_1());

        let expected = expect![[r#"
            lon: update fake_1

              043344a1c19619435e2b79cd42de6592308af0aa
            → 21386f9d14831b594048e1e4340ac7a300e312d6
        "#]];
        expected.assert_eq(&commit_message.to_string());
    }

    #[test]
    fn commit_message_multiple_updates() {
        let mut commit_message = CommitMessage::new();
        commit_message.add_summary("fake_1", summary_1());
        commit_message.add_summary("fake_2", summary_2());

        let expected = expect![[r#"
            lon: update

            • fake_1:
                043344a1c19619435e2b79cd42de6592308af0aa
              → 21386f9d14831b594048e1e4340ac7a300e312d6

            • fake_2:
                ad3bc97747c651e23fbc12c70a5849d3d8e9fdf4
              → 75962bcd89dcccc9fe125c9ab46377d6cd1ddb00
        "#]];
        expected.assert_eq(&commit_message.to_string());
    }

    #[test]
    fn commit_message_rev_list_single_update() {
        let mut commit_message = CommitMessage::new();
        commit_message.add_summary("fake_1", summary_rev_list_1());

        let expected = expect![[r#"
            lon: update fake_1

              043344a1c19619435e2b79cd42de6592308af0aa
            → 21386f9d14831b594048e1e4340ac7a300e312d6

            Last 4 commits:
              1ba800e readme: reorganize
              26244f0 readme: add section about bot
              c67d352 changelog: add entry about bot
              5de6d54 bot: init
        "#]];
        expected.assert_eq(&commit_message.to_string());
    }

    #[test]
    fn commit_message_rev_list_multiple_updates() {
        let mut commit_message = CommitMessage::new();
        commit_message.add_summary("fake_1", summary_rev_list_1());
        commit_message.add_summary("fake_2", summary_rev_list_2());

        let expected = expect![[r#"
            lon: update

            • fake_1:
                043344a1c19619435e2b79cd42de6592308af0aa
              → 21386f9d14831b594048e1e4340ac7a300e312d6

              Last 4 commits:
                1ba800e readme: reorganize
                26244f0 readme: add section about bot
                c67d352 changelog: add entry about bot
                5de6d54 bot: init

            • fake_2:
                6c1da4c913f0edf2835c3cc47c3889c36c05e6ca
              → 629f1e13eb7d09738538ba1b3c2ce35d9c1bef3e

              Last 3 commits:
                1ba800e emacs: remove native-comp-compiler-options-28.patch
                26244f0 readme: add section about bot
                6232894 .gitignore: ignore .env
        "#]];
        expected.assert_eq(&commit_message.to_string());
    }
}
