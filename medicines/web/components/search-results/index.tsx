import React from 'react';
import styled from 'styled-components';
import { black, mhraBlue10, mhraBlue80, white } from '../../styles/colors';
import {
  baseSpace,
  largePaddingSizeCss,
  tinyPaddingSizeCss,
} from '../../styles/dimensions';
import { baseFontSize, h2FontSize } from '../../styles/fonts';
import Pagination from './pagination';

const StyledDrugList = styled.section`
  .title {
    font-size: ${h2FontSize};
    padding: 0;
    margin: 0;
  }

  .no-of-results {
    padding-top: 0;
    padding-bottom: 30px;
    margin: 0;
  }

  .ema-message {
    padding: 0;
    margin: 0;
  }

  dl,
  dt,
  dd {
    list-style: none;
    padding: 0;
  }

  article {
    display: flex;
    background-color: ${mhraBlue10};
    padding: ${baseSpace};
    margin: ${baseSpace} 0;
    min-width: 1%;
    word-wrap: break-word;
  }

  dl p {
    margin: 0;
    padding: 0;
  }

  dd h3 {
    margin: 0;
  }

  dd a {
    text-decoration: none;
  }

  dt.left {
    flex: 0;
  }

  dt.left .icon {
    background-color: ${mhraBlue80};
    color: ${white};
    font-size: ${h2FontSize};
    font-weight: bold;
    padding: 10px 0;
    text-align: center;
    width: 70px;
  }

  dd.right {
    flex: 1;
    margin-left: 0;
    min-width: 1%;
    padding: 0 ${baseFontSize};
    word-wrap: break-word;
  }

  dd.right .title {
    font-size: ${h2FontSize};
    font-weight: bold;
    min-width: 1%;
    padding-bottom: ${tinyPaddingSizeCss};
    word-wrap: break-word;
  }

  dd.right .subtitle {
    font-size: ${baseFontSize};
    min-width: 1%;
    padding-bottom: 0.2rem;
    word-wrap: break-word;
  }

  dd.right .metadata {
    font-size: ${baseFontSize};
    min-width: 1%;
    padding-bottom: 0.1rem;
    word-wrap: break-word;
  }

  dd.right .context {
    font-size: ${h2FontSize};
    min-width: 1%;
    padding-top: ${largePaddingSizeCss};
    word-wrap: break-word;
  }

  em {
    font-weight: bold;
    font-style: normal;
  }
`;

export interface IDocument {
  activeSubstances: string[];
  context: string;
  created: string;
  docType: string;
  fileSize: string;
  name: string;
  product: string;
  url: string;
}

const emaWebsiteLink = () => (
  <a href="https://www.ema.europa.eu/en" target="_new">
    European Medicines Agency
  </a>
);

const searchResultsTitle = (
  showingResultsForTerm: string,
  noOfResults: number,
) => {
  return noOfResults === 0
    ? `There are no search results for ${showingResultsForTerm}`
    : `Showing results for ${showingResultsForTerm}`;
};

interface ISearchNumberingInformation {
  page: number;
  pageSize: number;
  totalResultCount: number;
  shownResultCount: number;
}

const searchResultsNumberingInformation = (
  numbering: ISearchNumberingInformation,
) => {
  const zero = (numbering.page - 1) * numbering.pageSize;
  const one = zero + 1;
  const last = zero + numbering.shownResultCount;

  return `${one} to ${last} of ${numbering.totalResultCount}`;
};

const normalizeDescription = (description: string): string => {
  const normalized = description
    .substr(0, 300) // Cut to 300 characters.
    .replace(/[^\w<>/\s…]/gi, '') // Remove non-word characters other than ellipsis & tags.
    .replace(/\s+/, ''); // Replace multi-spaces with one.

  if (/^\S/.test(description.substr(300))) {
    return normalized.replace(/\s+\S*$/, '') + '…'; // Add ellipsis.
  }

  return normalized;
};

function toSentenceCase(substance: string): string {
  return (
    (substance as string)
      .toLowerCase()
      .charAt(0)
      .toUpperCase() + substance.slice(1)
  );
}

const SearchResults = (props: {
  drugs: IDocument[];
  page: number;
  pageSize: number;
  resultCount: number;
  searchTerm: string;
  showingResultsForTerm: string;
}) => {
  return (
    <>
      <StyledDrugList>
        <div>
          <h1 className="title">
            {searchResultsTitle(
              props.showingResultsForTerm,
              props.drugs.length,
            )}
          </h1>
          {props.drugs.length > 0 && (
            <p className="no-of-results">
              {searchResultsNumberingInformation({
                page: props.page,
                pageSize: props.pageSize,
                shownResultCount: props.drugs.length,
                totalResultCount: props.resultCount,
              })}
            </p>
          )}
          <p className="ema-message">
            If the product information you are seeking does not appear below, it
            is possible that the product holds a central European license and
            its information may be available at the {emaWebsiteLink()} website.
          </p>
        </div>
        <dl>
          {props.drugs.length > 0 &&
            props.drugs.map((drug, i) => (
              <article key={i}>
                <dt className="left">
                  <p className="icon">{drug.docType.toUpperCase()}</p>
                </dt>
                <dd className="right">
                  {drug.product != null ? (
                    <a href={drug.url}>
                      <p className="title">{drug.product}</p>
                      <p className="subtitle">{drug.name}</p>
                    </a>
                  ) : (
                    <a href={drug.url}>
                      <p className="title">{drug.name}</p>
                    </a>
                  )}
                  <p className="metadata">File size: {drug.fileSize} KB</p>
                  <p className="metadata">Created: {drug.created}</p>
                  {drug.activeSubstances != null &&
                    drug.activeSubstances.length > 0 && (
                      <p className="metadata">
                        Active substances:{' '}
                        {drug.activeSubstances
                          .map(substance => toSentenceCase(substance))
                          .join(', ')}
                      </p>
                    )}
                  <p
                    className="context"
                    dangerouslySetInnerHTML={{
                      __html: normalizeDescription(drug.context),
                    }}
                  />
                </dd>
              </article>
            ))}
        </dl>
      </StyledDrugList>
      {props.resultCount > props.pageSize ? (
        <Pagination
          currentPage={props.page}
          pageSize={props.pageSize}
          resultCount={props.resultCount}
          searchTerm={props.searchTerm}
        />
      ) : (
        ''
      )}
    </>
  );
};

export default SearchResults;