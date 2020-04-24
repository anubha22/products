use crate::substance::Substance;
use juniper::GraphQLObject;
use search_client::{models::IndexResult, Search};

#[derive(GraphQLObject, Eq, Ord, PartialEq, PartialOrd)]
#[graphql(description = "A medical product containing active ingredients")]
pub struct Product {
    name: String,
    document_count: i32,
    documents: Vec<Document>,
}

impl Product {
    fn new(name: String, documents: Vec<Document>) -> Self {
        Self {
            name,
            document_count: documents.len() as i32,
            documents,
        }
    }

    fn add(&mut self, document: Document) {
        self.documents.push(document);
        self.document_count += 1;
    }
}

#[derive(GraphQLObject, Eq, Ord, PartialEq, PartialOrd)]
pub struct Document {
    product_name: Option<String>,
    active_substances: Option<Vec<String>>,
    title: Option<String>,
    highlights: Option<Vec<String>>,
    created: Option<String>,
    doc_type: Option<String>,
    file_bytes: Option<i32>,
    name: Option<String>,
    url: Option<String>,
}

impl From<IndexResult> for Document {
    fn from(r: IndexResult) -> Self {
        Self {
            product_name: r.product_name,
            active_substances: Some(r.substance_name),
            title: Some(r.title),
            created: r.created,
            doc_type: Some(r.doc_type),
            file_bytes: Some(r.metadata_storage_size),
            name: Some(r.file_name),
            url: Some(r.metadata_storage_path),
            highlights: match r.highlights {
                Some(a) => Some(a.content),
                _ => None,
            },
        }
    }
}

pub fn handle_doc(document: &IndexResult, products: &mut Vec<Product>) {
    match &document.product_name {
        Some(document_product_name) => {
            // Try to find an existing product.
            let existing_product = products
                .iter_mut()
                .find(|product| document_product_name == &product.name);

            match existing_product {
                Some(existing_product) => existing_product.add(document.to_owned().into()),
                None => products.push(Product::new(
                    document_product_name.to_owned(),
                    vec![document.to_owned().into()],
                )),
            }
        }
        None => {}
    }
}

pub async fn get_substance_with_products(
    substance_name: &str,
    client: &impl Search,
) -> Result<Substance, reqwest::Error> {
    let azure_result = client
        .filter_by_field("substance_name", substance_name)
        .await?;

    let mut products = Vec::<Product>::new();
    for document in azure_result.search_results {
        handle_doc(&document, &mut products);
    }

    products.sort();

    Ok(Substance::new(substance_name.to_string(), Some(products)))
}

pub async fn get_product(
    product_name: String,
    client: &impl Search,
) -> Result<Product, reqwest::Error> {
    let azure_result = client
        .filter_by_non_collection_field("product_name", &product_name)
        .await?;

    Ok(Product::new(
        product_name,
        azure_result
            .search_results
            .into_iter()
            .map(Into::<Document>::into)
            .collect(),
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    fn azure_result_factory(product_name: Option<String>) -> IndexResult {
        IndexResult {
            product_name,
            doc_type: "dummy".to_string(),
            created: Some("yes".to_string()),
            facets: Vec::new(),
            file_name: "README.markdown".to_string(),
            highlights: None,
            keywords: None,
            metadata_storage_name: "dummy".to_string(),
            metadata_storage_path: "/".to_string(),
            metadata_storage_size: 0,
            release_state: Some("solid".to_string()),
            rev_label: None,
            score: -0.0,
            substance_name: Vec::new(),
            suggestions: Vec::new(),
            title: "dummy's guide to medicines".to_string(),
        }
    }

    #[test]
    fn test_handle_doc_with_new_product() {
        let doc = azure_result_factory(Some("My Cool Product".to_string()));
        let mut products = Vec::<Product>::new();
        handle_doc(&doc, &mut products);
        assert_eq!(products.len(), 1);
        assert_eq!(products[0].name, "My Cool Product".to_string());
        assert_eq!(products[0].document_count, 1);
    }

    fn gimme_x_docs(x: i32) -> Vec<Document> {
        let mut docs: Vec<Document> = vec![];
        for i in 0..x {
            docs.push(azure_result_factory(Some("Craig's Cool Product".to_string())).into())
        }
        docs
    }

    #[test]
    fn test_handle_doc_with_existing_product() {
        let doc = azure_result_factory(Some("My Cool Product".to_string()));
        let mut products = Vec::<Product>::new();
        products.push(Product::new("My Cool Product".to_string(), gimme_x_docs(5)));
        handle_doc(&doc, &mut products);
        assert_eq!(products.len(), 1);
        assert_eq!(products[0].name, "My Cool Product".to_string());
        assert_eq!(products[0].document_count, 6);
    }

    #[test]
    fn test_handle_doc_with_no_product_name() {
        let doc = azure_result_factory(None);
        let mut products = Vec::<Product>::new();
        handle_doc(&doc, &mut products);
        assert_eq!(products.len(), 0);
    }

    #[test]
    fn test_sort_products() {
        let mut products = Vec::<Product>::new();
        products.push(Product::new("B".to_owned(), gimme_x_docs(1)));
        products.push(Product::new("C".to_owned(), gimme_x_docs(1)));
        products.push(Product::new("A".to_owned(), gimme_x_docs(1)));
        products.sort();
        assert_eq!(products[0].name, "A");
        assert_eq!(products[1].name, "B");
        assert_eq!(products[2].name, "C");
    }
}
