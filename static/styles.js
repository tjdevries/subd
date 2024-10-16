import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

/*
Pineapple Fever: A song that expresses a deep and fun craving for pineapple pizza, capturing the sweetness, tanginess, and the joy of sharing this flavor sensation with someone special. The lyrics describe a tropical journey, a love that fuses culinary and emotional flavors, and the irresistible allure of pineapple.
*/

// Initialize Mermaid for chart rendering
mermaid.initialize({ startOnLoad: true });

console.log('Mermaid initialized for charts.');

// Example Mermaid chart for flavor ride:
const chartDefinition = `
graph TD;
  Pizza -->|one bite| Flavor_Bliss[(Tropical Bliss)];
  Flavor_Bliss --> Pineapple[(Pineapple)];
  Flavor_Bliss --> Dough[(Dough)];
  Pineapple -->|tangy| PalmTrees[(Palm Trees)];
  Dough -->|delicious| PalmTrees;
  
  subgraph FlavorRide 
      PalmTrees 
  end
`;

mermaid.render("flavorRideDiagram", chartDefinition, function(svgCode) {
  document.getElementById("flavor-ride").innerHTML = svgCode;
  console.log('Mermaid chart rendered.');
});

// Create a Three.js Scene for animated visual experiences
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();

renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

console.log('Three.js scene initiated.');

// Create a rotating pineapple
const geometry = new THREE.ConeGeometry(5, 20, 32);
const material = new THREE.MeshBasicMaterial({ color: 0xffff00 }); // Yellow color to mimic pineapple
const pineapple = new THREE.Mesh(geometry, material);
scene.add(pineapple);

pineapple.position.y = 10;

camera.position.z = 50;

function animate() {
  requestAnimationFrame(animate);

  pineapple.rotation.x += 0.01;
  pineapple.rotation.y += 0.01;

  renderer.render(scene, camera);
  console.log('Pineapple rotating.');
}

animate();

// Using D3.js to create a fun data visualization of "Pineapple Fever" loves
const data = [10, 20, 30, 40]; // Arbitrary example data

const svg = D3.select('body').append('svg')
  .attr('width', window.innerWidth)
  .attr('height', 100);

const xScale = D3.scaleBand()
  .domain(data.map((value, index) => index))
  .range([0, window.innerWidth])
  .padding(0.1);

const yScale = D3.scaleLinear()
  .domain([0, D3.max(data)])
  .range([100, 0]);

svg.selectAll('.bar')
  .data(data)
  .enter()
  .append('rect')
  .classed('bar', true)
  .attr('x', (d, i) => xScale(i))
  .attr('y', d => yScale(d))
  .attr('width', xScale.bandwidth())
  .attr('height', d => 100 - yScale(d))
  .attr('fill', '#ffff00');

console.log('D3.js bar chart created.');