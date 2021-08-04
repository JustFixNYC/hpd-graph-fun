import { getHTMLElement } from '@justfixnyc/util';
import ForceGraph, { GraphData, LinkObject, NodeObject } from 'force-graph';

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
    nodes: p.nodes.map((node): NodeObject => ({
      id: node.id,
      name: 'Name' in node.value ? node.value.Name : node.value.BizAddr,
      color: 'Name' in node.value ? 'crimson' : 'gray',
      val: 10,
    })),
    links: p.edges.map((edge): LinkObject => ({
      source: edge.from,
      name: `${edge.reg_contacts} HPD registration${edge.reg_contacts === 1 ? '' : 's'}`,
      color: getEdgeColor(edge),
      target: edge.to,
    })),
  };
}

async function main() {
  const graphEl = getHTMLElement('div', '#graph');
  const messageEl = getHTMLElement('p', '#message');
  const searchForm = getHTMLElement("form", "#search-form");
  const searchInput = getHTMLElement("input", "#search-input");

  const filename = 'portfolio.json';
  const portfolioReq = await fetch(filename);

  searchInput.value = "";

  if (!portfolioReq.ok) {
    messageEl.textContent = `Got HTTP ${portfolioReq.status} when trying to retrive ${filename}!`;
    return;
  }

  const portfolio: Portfolio = await portfolioReq.json();
  const selectedNodes = new Set<NodeObject>();

  document.title = portfolio.title;
  getHTMLElement('h1', '').textContent = document.title;

  messageEl.textContent = `Loaded portfolio with ${portfolio.nodes.length} nodes and ${portfolio.edges.length} edges.`;

  const graphData = portfolioToGraphData(portfolio);

  const graph = ForceGraph()(graphEl).graphData(graphData).nodeColor(node => {
    return selectedNodes.has(node) ? "blue" : node.color;
  });

  searchForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const query = searchInput.value.toUpperCase();

    selectedNodes.clear();

    if (!query) {
      messageEl.textContent = "";
      graph.zoomToFit(500, 20, node => true);
      return;
    }

    for (let node of graphData.nodes) {
      if (node.name.includes(query)) {
        selectedNodes.add(node);
      }
    }

    if (selectedNodes.size > 0) {
      messageEl.textContent = `Found ${selectedNodes.size} node(s) with the text "${query}".`;
      if (selectedNodes.size === 1) {
        const bbox = graph.getGraphBbox(node => selectedNodes.has(node));
        graph.centerAt(bbox.x[0], bbox.y[0], 500);
      } else {
        graph.zoomToFit(500, 20, node => selectedNodes.has(node));
      }
    } else {
      messageEl.textContent = `Unable to find any nodes with the text "${query}".`;
    }
  });
}

window.addEventListener("load", main);
