import ForceGraph, { GraphData } from 'force-graph';

function portfolioToGraphData(p: Portfolio): GraphData {
  return {
    nodes: p.nodes.map(node => ({
      id: node.id,
      name: 'Name' in node.value ? node.value.Name : node.value.BizAddr,
      color: 'Name' in node.value ? 'pink' : 'gray',
      val: 10,
    })),
    links: p.edges.map(edge => ({
      source: edge.from,
      target: edge.to,
    })),
  };
}

async function main() {
  const graphEl = document.createElement("div");
  document.body.appendChild(graphEl);

  const filename = 'portfolio.json';
  const portfolioReq = await fetch(filename);

  if (!portfolioReq.ok) {
    alert(`Got HTTP ${portfolioReq.status} when trying to retrive ${filename}`);
    return;
  }

  const portfolio: Portfolio = await portfolioReq.json();
  const graphData = portfolioToGraphData(portfolio);
  const graph = ForceGraph()(graphEl).linkDirectionalParticles(2).graphData(graphData);
}

window.addEventListener("load", main);
