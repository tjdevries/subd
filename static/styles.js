// This file includes animations and charts for the HTML provided
import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

// Configuration for Mermaid.js
mermaid.initialize({
  startOnLoad: true
});

console.log('Mermaid.js initialized');

// Animate the header with Three.js
function animateHeader() {
  const header = document.querySelector('.header-container');
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
  const renderer = new THREE.WebGLRenderer();
  renderer.setSize(window.innerWidth, window.innerHeight);
  document.body.appendChild(renderer.domElement);

  const geometry = new THREE.BoxGeometry();
  const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
  const cube = new THREE.Mesh(geometry, material);
  scene.add(cube);

  camera.position.z = 5;

  function animate() {
    requestAnimationFrame(animate);
    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;
    renderer.render(scene, camera);
  }
  animate();
  console.log('Three.js animated header initialized');
}

// Display chart with Mermaid.js
function displayMermaidChart() {
  const chartsContainer = document.createElement('div');
  chartsContainer.className = 'charts-container';
  document.body.appendChild(chartsContainer);

  const chart = document.createElement('div');
  chart.className = 'mermaid';
  chart.innerHTML = `
    graph TD
      A[Cellphone Speaker] --> B[Humming Spirals]
      A --> C[Digital Whispers]
      B --> D[Radio Whispers]
      C --> E[Dance of Waves]
      D --> F[Interference Tango]
  `;
  chartsContainer.appendChild(chart);

  mermaid.init();
  console.log('Mermaid chart displayed');
}

// Animate song lyrics section with D3.js
function animateSongLyrics() {
  const sections = document.querySelectorAll('.song-lyrics p');
  sections.forEach(section => {
    D3.select(section)
      .transition()
      .duration(2000)
      .style('color', 'red')
      .style('font-size', '20px');
  });
  console.log('Song lyrics animation applied');
}

// Initialize all animations and charts
window.onload = function() {
  animateHeader();
  displayMermaidChart();
  animateSongLyrics();
  console.log('All animations and charts initialized');
};