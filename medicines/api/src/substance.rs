use crate::{document::Document, product::Product};
use search_client::{
    models::{DocTypeParseError, IndexResults},
    Search,
};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub struct Substance {
    name: String,
    products: Vec<Product>,
}

impl Substance {
    pub fn new(name: String, products: Vec<Product>) -> Self {
        Self { name, products }
    }
}

#[juniper::graphql_object]
#[graphql(description = "An active ingredient found in medical products")]
impl Substance {
    fn name(&self) -> &str {
        &self.name
    }

    fn products(&self) -> &Vec<Product> {
        &self.products
    }
}

pub async fn get_substances_starting_with_letter(
    client: &impl Search,
    letter: char,
) -> anyhow::Result<Vec<Substance>> {
    let upper_letter = letter.to_ascii_uppercase();

    let azure_result = client
        .filter_by_collection_field("facets", &upper_letter.to_string())
        .await?;

    Ok(format_search_results(azure_result, upper_letter)?)
}

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct SubstanceName(String);

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct ProductName(String);

fn format_search_results(
    results: IndexResults,
    letter: char,
) -> Result<Vec<Substance>, DocTypeParseError> {
    // Using a BTreeMap (instead of HashMap) so that the keys are sorted alphabetically.
    let mut substances: BTreeMap<SubstanceName, BTreeMap<ProductName, Vec<Document>>> =
        BTreeMap::new();

    let letter_string = letter.to_string();

    results
        .search_results
        .into_iter()
        .filter(|result| result.facets.iter().any(|s| s == &letter_string))
        .filter_map(|result| {
            let doc: Document = result.into();

            let substance = doc
                .substances()
                .find(|s| s.starts_with(letter))?
                .to_string();

            let product = doc.product_name()?.to_string();

            Some((SubstanceName(substance), ProductName(product), doc))
        })
        .for_each(|(substance, product, doc)| {
            add_product_for_substance(&mut substances, substance, product, doc);
        });

    let substances_vec = substances
        .into_iter()
        .map(|(SubstanceName(substance), products)| {
            let products_vec = products
                .into_iter()
                .map(|(ProductName(name), docs)| Product::new(name, Some(docs)))
                .collect();

            Substance::new(substance, products_vec)
        })
        .collect();

    Ok(substances_vec)
}

fn add_product_for_substance(
    substances: &mut BTreeMap<SubstanceName, BTreeMap<ProductName, Vec<Document>>>,
    substance: SubstanceName,
    product: ProductName,
    doc: Document,
) {
    match substances.get_mut(&substance) {
        None => {
            let mut products = BTreeMap::new();
            products.insert(product, vec![doc]);
            substances.insert(substance, products);
        }
        Some(mut products) => add_document_for_product(&mut products, product, doc),
    }
}

fn add_document_for_product(
    products: &mut BTreeMap<ProductName, Vec<Document>>,
    name: ProductName,
    document: Document,
) {
    match products.get_mut(&name) {
        Some(docs) => {
            docs.push(document);
        }
        None => {
            products.insert(name, vec![document]);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use search_client::models::{DocumentType, IndexResult, IndexResults};

    fn index_result(product_name: &str, substance_name: &[&str], facets: &[&str]) -> IndexResult {
        IndexResult {
            doc_type: DocumentType::Spc,
            file_name: "CON1587463572172".into(),
            metadata_storage_name: "4e99070c7e5d3682675b2becd972ec44ef35b20c".into(),
            metadata_storage_path: "https://mhraproductsnonprod.blob.core.windows.net/docs/4e99070c7e5d3682675b2becd972ec44ef35b20c".into(),
            product_name: Some(product_name.into()),
            substance_name: substance_name.iter().map(|&s|s.into()).collect(),
            title: "spc-doc_PL 16363-0365.pdf".into(),
            created: None,
            facets: facets.iter().map(|&s|s.into()).collect(),
            keywords: Some("".into()),
            metadata_storage_size: 107842,
            release_state: None,
            rev_label: None,
            suggestions: vec![],
            score: 1.0,
            highlights: None
        }
    }

    #[test]
    fn formats_search_results_containing_multiple_products() {
        let letter = 'Z';

        let zonismade_25mg = index_result(
            "ZONISAMIDE ARISTO 25 MG HARD CAPSULES",
            &["ZONISAMIDE"],
            &[
                "Z",
                "Z, ZONISAMIDE",
                "Z, ZONISAMIDE, ZONISAMIDE ARISTO 25 MG HARD CAPSULES",
            ],
        );
        let zonismade_50mg = index_result(
            "ZONISAMIDE ARISTO 50 MG HARD CAPSULES",
            &["ZONISAMIDE"],
            &[
                "Z",
                "Z, ZONISAMIDE",
                "Z, ZONISAMIDE, ZONISAMIDE ARISTO 50 MG HARD CAPSULES",
            ],
        );
        let zonismade_50mg_repeat = index_result(
            "ZONISAMIDE ARISTO 50 MG HARD CAPSULES",
            &["ZONISAMIDE"],
            &[
                "Z",
                "Z, ZONISAMIDE",
                "Z, ZONISAMIDE, ZONISAMIDE ARISTO 50 MG HARD CAPSULES",
            ],
        );
        let zolmitriptan = index_result(
            "ZOMIG RAPIMELT 2.5 MG ORODISPERSIBLE TABLETS",
            &["ZOLMITRIPTAN"],
            &[
                "Z",
                "Z, ZOLMITRIPTAN",
                "Z, ZOLMITRIPTAN, ZOMIG RAPIMELT 2.5 MG ORODISPERSIBLE TABLETS",
            ],
        );

        let results = IndexResults {
            search_results: vec![
                zonismade_25mg.clone(),
                zonismade_50mg.clone(),
                zonismade_50mg_repeat.clone(),
                zolmitriptan.clone(),
            ],
            context: "https://mhraproductsnonprod.search.windows.net/indexes(\'products-index\')/$metadata#docs(*)".into(),
            count: None
        };

        let zon50: Vec<Document> = vec![zonismade_50mg.into(), zonismade_50mg_repeat.into()];
        let zon25: Vec<Document> = vec![zonismade_25mg.into()];
        let zol: Vec<Document> = vec![zolmitriptan.into()];

        let formatted = format_search_results(results, letter).unwrap();

        let expected = vec![
            Substance::new(
                "ZOLMITRIPTAN".into(),
                vec![Product::new(
                    "ZOMIG RAPIMELT 2.5 MG ORODISPERSIBLE TABLETS".into(),
                    Some(zol),
                )],
            ),
            Substance::new(
                "ZONISAMIDE".into(),
                vec![
                    Product::new("ZONISAMIDE ARISTO 25 MG HARD CAPSULES".into(), Some(zon25)),
                    Product::new("ZONISAMIDE ARISTO 50 MG HARD CAPSULES".into(), Some(zon50)),
                ],
            ),
        ];

        assert_eq!(formatted, expected);
    }

    #[test]
    fn formats_products_that_contain_multiple_substances() {
        let letter = 'Z';

        let index_result = index_result(
            "LAMIVUDINE/ZIDOVUDINE 150 MG/300 MG FILM-COATED TABLETS",
            &["LAMIVUDINE", "ZIDOVUDINE"],
            &[
                "L",
                "L, LAMIVUDINE",
                "L, LAMIVUDINE, LAMIVUDINE/ZIDOVUDINE 150 MG/300 MG FILM-COATED TABLETS",
                "Z",
                "Z, ZIDOVUDINE",
                "Z, ZIDOVUDINE, LAMIVUDINE/ZIDOVUDINE 150 MG/300 MG FILM-COATED TABLETS",
            ],
        );
        let document: Document = index_result.clone().into();

        let results = IndexResults {
            search_results: vec![
                index_result,
            ],
            context: "https://mhraproductsnonprod.search.windows.net/indexes(\'products-index\')/$metadata#docs(*)".into(),
            count: None
        };

        let formatted = format_search_results(results, letter).unwrap();

        let expected = vec![Substance::new(
            "ZIDOVUDINE".into(),
            vec![Product::new(
                "LAMIVUDINE/ZIDOVUDINE 150 MG/300 MG FILM-COATED TABLETS".into(),
                Some(vec![document]),
            )],
        )];

        assert_eq!(formatted, expected);
    }
}
