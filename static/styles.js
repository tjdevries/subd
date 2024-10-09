//// styles.js //
import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

// Initialize mermaid for chart generation
mermaid.initialize({ startOnLoad: true });

// Function to create a three.js animated background
function createThreeJSBackground() {
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
  const renderer = new THREE.WebGLRenderer();
  renderer.setSize(window.innerWidth, window.innerHeight);
  document.body.appendChild(renderer.domElement);

  const geometry = new THREE.TorusGeometry(10, 3, 16, 100);
  const material = new THREE.MeshBasicMaterial({ color: 0xff6347, wireframe: true });
  const torus = new THREE.Mesh(geometry, material);
  scene.add(torus);

  camera.position.z = 50;

  function animate() {
    requestAnimationFrame(animate);

    torus.rotation.x += 0.01;
    torus.rotation.y += 0.01;
    console.log('Animating Torus');

    renderer.render(scene, camera);
  }

  animate();
}

// Function to create a D3 animated chart
function createD3Chart() {
  const data = [10, 20, 30, 40, 50];
  const svg = D3.select('body').append('svg')
    .attr('width', 500)
    .attr('height', 100);

  svg.selectAll('rect')
    .data(data)
    .enter()
    .append('rect')
    .attr('x', (d, i) => i * 30)
    .attr('y', 20)
    .attr('width', 20)
    .attr('height', d => d)
    .attr('fill', 'tomato')
    .transition()
    .duration(1000)
    .attr('height', d => d * 2)
    .on('end', () => console.log('D3 Chart Animated'));
}

// Function to render a Mermaid chart
function renderMermaidChart() {
  const graphDefinition = `graph TD;
  A[Sing a song] --> B[Let it echo];
  B --> C{Infinite loops};
  C -->|Yes| D[Continue singing];
  C -->|No| E[Finish song];`;

  const graphEl = document.createElement('div');
  graphEl.classList.add('mermaid');
  graphEl.innerHTML = graphDefinition;
  document.body.appendChild(graphEl);
  mermaid.init(undefined, graphEl);
  console.log('Mermaid Chart Rendered');
}

// Execute the animations and chart rendering
createThreeJSBackground();
createD3Chart();
renderMermaidChart();

console.log('All animations initiated');