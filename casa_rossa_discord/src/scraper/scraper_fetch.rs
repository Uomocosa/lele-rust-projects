use crate::scraper;
use html_scraper::{Html, Selector};

pub async fn fetch(ms: &scraper::MenuScraper) -> crate::Result<Vec<scraper::ImageEntry>> {
    let url = format!(
        "{}/index.php?option=com_dropbox&view=dropbox&format=raw&id=1",
        ms.base_url
    );

    let html = ms.client.get(&url).send().await?.text().await?;

    let entries = {
        let document = Html::parse_document(&html);
        let img_selector = Selector::parse(".dropbox_pic_wrapper img")
            .map_err(|e| crate::Error::Scrape(format!("CSS selector parse error: {e:?}")))?;

        document
            .select(&img_selector)
            .map(|element| {
                let longdesc = element
                    .attr("longdesc")
                    .ok_or_else(|| {
                        crate::Error::Scrape("missing longdesc attribute on image".into())
                    })?
                    .to_string();
                let filename = element.attr("name").unwrap_or("menu.jpg").to_string();
                Ok((filename, longdesc))
            })
            .collect::<crate::Result<Vec<_>>>()?
    };

    let mut images = Vec::new();

    for (filename, full_url) in entries {
        let data = ms
            .client
            .get(&full_url)
            .send()
            .await?
            .bytes()
            .await?
            .to_vec();

        images.push(scraper::ImageEntry { filename, data });
    }

    Ok(images)
}

#[cfg(test)]
mod tests {
    use html_scraper::{Html, Selector};

    #[test]
    fn test_parse_dropbox_html() {
        let html = r#"<div class='dropbox_pic_wrapper'>
            <a href="/images/fullsize_file.jpg">
              <img class="dropbox_pictures_1"
                   name="test_menu.jpg"
                   longdesc="http://example.com/test_full.jpg" /></a></div>"#;
        let document = Html::parse_document(html);
        let img_selector = Selector::parse(".dropbox_pic_wrapper img").unwrap();
        let mut found = false;
        for element in document.select(&img_selector) {
            assert_eq!(element.attr("name").unwrap(), "test_menu.jpg");
            assert_eq!(
                element.attr("longdesc").unwrap(),
                "http://example.com/test_full.jpg"
            );
            found = true;
        }
        assert!(found);
    }
}
