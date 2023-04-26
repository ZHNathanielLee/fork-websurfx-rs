use std::collections::HashMap;

use super::{
    aggregation_models::{RawSearchResult, SearchResult, SearchResults},
    user_agent::random_user_agent,
};

use crate::engines::{duckduckgo, searx};

// A function that aggregates all the scraped results from the above upstream engines and
// then removes duplicate results and if two results are found to be from two or more engines
// then puts their names together to show the results are fetched from these upstream engines
// and then removes all data from the HashMap and puts into a struct of all results aggregated
// into a vector and also adds the query used into the struct this is neccessory because otherwise
// the search bar in search remains empty if searched from the query url
//
// For Example:
//
// If you search from the url like *https://127.0.0.1/search?q=huston* then the search bar should
// contain the word huston and not remain empty.
pub async fn aggregate(
    query: &str,
    page: Option<u32>,
) -> Result<SearchResults, Box<dyn std::error::Error>> {
    let user_agent: String = random_user_agent();
    let mut result_map: HashMap<String, RawSearchResult> = HashMap::new();

    let ddg_map_results: HashMap<String, RawSearchResult> =
        duckduckgo::results(query, page, &user_agent).await?;
    let searx_map_results: HashMap<String, RawSearchResult> =
        searx::results(query, page, &user_agent).await?;

    result_map.extend(ddg_map_results);

    searx_map_results.into_iter().for_each(|(key, value)| {
        result_map
            .entry(key)
            .and_modify(|result| {
                result.add_engines(value.clone().engine());
            })
            .or_insert_with(|| -> RawSearchResult {
                RawSearchResult::new(
                    value.title.clone(),
                    value.visiting_url.clone(),
                    value.description.clone(),
                    value.engine.clone(),
                )
            });
    });

    Ok(SearchResults::new(
        result_map
            .into_iter()
            .map(|(key, value)| {
                SearchResult::new(
                    value.title,
                    value.visiting_url,
                    key,
                    value.description,
                    value.engine,
                )
            })
            .collect(),
        query.to_string(),
    ))
}
