use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use url::form_urlencoded;

const DEFAULT_PAGE_SIZE: u32 = 15;

#[derive(Builder, Debug)]
pub struct PageOptions {
    #[builder(setter(into), default = "DEFAULT_PAGE_SIZE")]
    page_size: u32,
}

impl PageOptions {
    pub fn builder() -> PageOptionsBuilder {
        PageOptionsBuilder::default()
    }

    pub fn serialize(&self) -> Option<String> {
        let page_size = self.page_size.to_string();
        let params = vec![("linked_partitioning", "true"), ("page_size", &page_size)];

        let encoded: String = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();
        Some(encoded)
    }
}

impl Default for PageOptions {
    fn default() -> Self {
        PageOptionsBuilder::default()
            .build()
            .expect("PageOptionsBuilder should always succeed with defaults")
    }
}

/// Paginated response
#[derive(Serialize, Deserialize, Debug)]
pub struct Page<T> {
    pub collection: Vec<T>,
    pub next_href: Option<String>,
}

impl<T> Page<T> {
    #[allow(dead_code)]
    pub fn next_query(&self) -> crate::error::Result<Option<std::collections::HashMap<String, String>>> {
        match &self.next_href {
            Some(href) => {
                let url = url::Url::parse(href)?;
                let next_query: std::collections::HashMap<String, String> =
                    url.query_pairs().into_owned().collect();
                Ok((!next_query.is_empty()).then_some(next_query))
            }
            None => Ok(None),
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.collection.is_empty()
    }
}
