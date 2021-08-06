type PortfolioNode = {
  id: number,
  value: { Name: string }|{ BizAddr: string },
};

type PortfolioEdge = {
  from: number,
  to: number,
  reg_contacts: number,
  is_bridge: boolean,
};

type Portfolio = {
  title: string,
  nodes: PortfolioNode[],
  edges: PortfolioEdge[],
};
