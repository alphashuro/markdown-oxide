use crate::config::Settings;

pub struct Link {
    pub refname: String,
    pub display: Option<String>,
}

impl Link {
    pub fn as_markdown_link(&self, settings: &Settings) -> String {
        let ext = if settings.include_md_extension_md_link {
            ".md"
        } else {
            ""
        };

        let format_link = |name: &str, suffix: &str| {
            if name.contains(' ') || suffix.contains(' ') {
                format!("<{}{}{}>", name, ext, suffix)
            } else {
                format!("{}{}{}", name, ext, suffix)
            }
        };

        // Handle block links foobar#^123 -> foobar.md#^123
        let link_ref_text = if let Some(pos) = self.refname.find("#^") {
            let (name, suffix) = self.refname.split_at(pos);

            if settings.link_filenames_only {
                format_link(name, "")
            } else {
                format_link(name, suffix)
            }
        // Handle headings links foobar#myheading -> foobar.md#myheading
        } else if let Some(pos) = self.refname.find('#') {
            let (name, suffix) = self.refname.split_at(pos);

            if settings.link_filenames_only {
                format_link(name, "")
            } else {
                format_link(name, suffix)
            }
        } else {
            // default case foobar -> foobar.md
            format_link(self.refname.as_str(), "")
        };

        format!(
            "[{}]({})",
            self.display.clone().unwrap_or("".to_string()),
            link_ref_text
        )
    }

    pub fn as_wikilink(&self, settings: &Settings) -> String {
        let ext = if settings.include_md_extension_wikilink {
            ".md"
        } else {
            ""
        };

        format!(
            "[[{}{}{}]]",
            self.refname,
            ext,
            self.display
                .clone()
                .map(|display| format!("|{}", display))
                .unwrap_or("".to_string())
        )
    }

    pub fn as_link_text(&self, settings: &Settings) -> String {
        match settings.use_markdown_links {
            Some(false) => self.as_wikilink(settings),
            _ => self.as_markdown_link(settings),
        }
    }
}
