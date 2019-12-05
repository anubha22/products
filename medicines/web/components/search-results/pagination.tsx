import React from 'react';

const getPaginationGroups = (pageCount: number, currentPage: number) => {
  let firstGroup: number[] = [1];
  let middleGroup: number[] = [];
  let lastGroup: number[] = [pageCount];

  switch (currentPage) {
    case 1:
    case 2:
    case 3:
      firstGroup = Array(currentPage + 1)
        .fill(1)
        .map((_, i) => i + 1);
      break;
    case pageCount - 2:
    case pageCount - 1:
    case pageCount:
      lastGroup = Array(pageCount - currentPage + 2)
        .fill(1)
        .map((_, i) => currentPage - 1 + i);
      break;
    default:
      middleGroup = [currentPage - 1, currentPage, currentPage + 1];
      break;
  }
  return {
    firstGroup,
    middleGroup,
    lastGroup,
  };
};

const Pagination = (props: {
  searchTerm: string;
  resultCount: number;
  pageSize: number;
  currentPage: number;
}) => {
  const paginationHref = (p: number) =>
    `/?search=${props.searchTerm}&page=${p}`;

  const pageCount = Math.floor(props.resultCount / props.pageSize) + 1;
  const { firstGroup, middleGroup, lastGroup } = getPaginationGroups(
    pageCount,
    props.currentPage,
  );

  const mapper = (p: number) => {
    if (p === props.currentPage) {
      return <li key={p}>{p}</li>;
    }

    return (
      <li key={p}>
        <a href={paginationHref(p)}>{p}</a>
      </li>
    );
  };

  return (
    <nav>
      <ul>
        {firstGroup.map(mapper)}
        {middleGroup.length > 0 ? <li>...</li> : ''}
        {middleGroup.map(mapper)}
        <li>...</li>
        {lastGroup.map(mapper)}
      </ul>
    </nav>
  );
};

export default Pagination;
