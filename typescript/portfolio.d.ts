type PortfolioNode = {
  id: number,
  value: { Name: string }|{ BizAddr: string },
};

type PortfolioEdge = {
  from: number,
  to: number,
  reg_contacts: number,
};

type Portfolio = {
  title: string,
  nodes: PortfolioNode[],
  edges: PortfolioEdge[],
};
