import { getHTMLElement } from '@justfixnyc/util';
import ForceGraph, { GraphData } from 'force-graph';

function getEdgeColor(edge: PortfolioEdge): string {
  if (edge.reg_contacts === 1) {
    return 'lightgray'
  } else if (edge.reg_contacts < 10) {
    return 'gray';
  }
  return 'black';
}

function portfolioToGraphData(p: Portfolio): GraphData {
  return {
    nodes: p.nodes.map(node => ({
      id: node.id,
      name: 'Name' in node.value ? node.value.Name : node.value.BizAddr,
      color: 'Name' in node.value ? 'crimson' : 'gray',
      val: 10,
    })),
    links: p.edges.map(edge => ({
      source: edge.from,
      name: `${edge.reg_contacts} HPD registration${edge.reg_contacts === 1 ? '' : 's'}`,
      color: getEdgeColor(edge),
      target: edge.to,
    })),
  };
}

async function main() {
  const graphEl = getHTMLElement('div', '#graph');

  const filename = 'portfolio.json';
  const portfolioReq = await fetch(filename);

  if (!portfolioReq.ok) {
    alert(`Got HTTP ${portfolioReq.status} when trying to retrive ${filename}`);
    return;
  }

  const portfolio: Portfolio = await portfolioReq.json();

  document.title = portfolio.title;
  getHTMLElement('h1', '').textContent = document.title;

  const graphData = portfolioToGraphData(portfolio);

  const graph = ForceGraph()(graphEl).linkDirectionalParticles(2).graphData(graphData);
}

window.addEventListener("load", main);
