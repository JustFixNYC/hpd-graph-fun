import { assertNotNull, getHTMLElement } from '@justfixnyc/util';
import ForceGraph, { GraphData, LinkObject, NodeObject } from 'force-graph';

const NAME_COLOR = 'crimson';

const BIZADDR_COLOR = 'gray';

const LEGEND_HTML = `
<details>
  <summary>Legend</summary>
  <p>Each name is a ${coloredLabel(NAME_COLOR)} node.</p>
  <p>Each business address is a  ${coloredLabel(BIZADDR_COLOR)} node.</p>
  <p>A name node and address node are connected via an edge if at least one HPD registration contact contains both (i.e., if the name is associated with the address).</p>
  <p>The weight of an edge corresponds to the number of HPD registration contacts it has.</p>
  <p>The edge is a dashed line if it corresponds to only one HPD contact registration <em>and</em> is a local bridge.</p>
  <p>Clicking on an edge will open one (of possibly many!) associated buildings in <em>Who Owns What</em>.</p>
</details>
`;

function wowLink(bbl: string): string {
  return `https://whoownswhat.justfix.nyc/en/bbl/${bbl}`;
}

function coloredLabel(background: string, color: string = 'white'): string {
  return `<span style="background-color: ${background}; color: white">${background}</span>`;
}

function getEdgeColor(edge: PortfolioEdge): string {
  if (edge.reg_contacts === 1) {
    return 'lightgray'
  } else if (edge.reg_contacts < 10) {
    return 'gray';
  }
  return 'black';
}

function getEdgeLabel(edge: PortfolioEdge): string {
  const parts = [
    `${edge.reg_contacts} HPD registration${edge.reg_contacts === 1 ? '' : 's'} ` +
    `(e.g. BBL ${edge.bbl})`
  ];

  if (edge.is_bridge) {
    parts.push('Local bridge');
  }

  return parts.join('<br>');
}

function portfolioToGraphData(p: Portfolio): GraphData {
  return {
    nodes: p.nodes.map((node): NodeObject => ({
      id: node.id,
      name: 'Name' in node.value ? node.value.Name : node.value.BizAddr,
      color: 'Name' in node.value ? NAME_COLOR : BIZADDR_COLOR,
      val: 10,
    })),
    links: p.edges.map((edge): LinkObject => ({
      source: edge.from,
      name: getEdgeLabel(edge),
      color: getEdgeColor(edge),
      edge,
      target: edge.to,
    })),
  };
}

function main() {
  const graphEl = getHTMLElement('div', '#graph');
  const messageEl = getHTMLElement('p', '#message');
  const searchForm = getHTMLElement("form", "#search-form");
  const searchInput = getHTMLElement("input", "#search-input");
  const portfolio: Portfolio = JSON.parse(assertNotNull(getHTMLElement("script", "#portfolio").textContent));
  const legendEl = document.createElement('div');

  legendEl.id = 'legend';
  legendEl.innerHTML = LEGEND_HTML;
  document.body.appendChild(legendEl);

  searchInput.value = "";

  const selectedNodes = new Set<NodeObject>();

  document.title = portfolio.title;
  getHTMLElement('h1', '').textContent = document.title;

  messageEl.textContent = `Loaded portfolio with ${portfolio.nodes.length} nodes and ${portfolio.edges.length} edges.`;

  const graphData = portfolioToGraphData(portfolio);

  const graph = ForceGraph()(graphEl)
    .graphData(graphData)
    .nodeColor(node => {
      return selectedNodes.has(node) ? "blue" : node.color;
    })
    .linkLineDash(
      link => link.edge.is_bridge && link.edge.reg_contacts === 1 ? [5, 5] : null
    )
    .onLinkClick(link => {
      window.open(wowLink(link.edge.bbl), '_blank');
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
