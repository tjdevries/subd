// Import necessary libraries
import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

console.log('Loaded libraries: THREE.js, D3.js, mermaid.js');

// Initialize Three.js for 3D animations
function initThreeJSAnimation() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);

    camera.position.z = 5;
    
    console.log('Cube created');

    // Function for animation
    function animate() {
        requestAnimationFrame(animate);
        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;
        renderer.render(scene, camera);
        console.log('Cube rotated');
    }

    animate();
}

initThreeJSAnimation();

// Setup Mermaid diagrams
mermaid.initialize({ startOnLoad: true });

// Create a Mermaid chart
function initMermaidChart() {
    const graphDefinition = `graph TD;
      A[Start] --> B{Where Did It Go};
      B -->|Various Choices| C[Explore];
      C --> D[End];
    `;

    const graphDiv = document.createElement('div');
    graphDiv.class = 'mermaid';
    graphDiv.innerHTML = graphDefinition;
    document.body.appendChild(graphDiv);
    console.log('Mermaid graph created');
}

initMermaidChart();

// Basic D3.js chart example
function initD3Chart() {
    const dataset = [5, 10, 15, 20, 25];
    const svg = D3.select('body').append('svg')
        .attr('width', 300)
        .attr('height', 200);

    svg.selectAll('rect')
        .data(dataset)
        .enter()
        .append('rect')
        .attr('x', (d, i) => i * 30)
        .attr('y', d => 200 - d * 5)
        .attr('width', 25)
        .attr('height', d => d * 5)
        .attr('fill', 'teal');

    console.log('D3 chart created');
}

initD3Chart();

// Animation and logs influenced by song concept 'Where Did It Go'
console.log('Begin animations inspired by concept: "Where Did It Go"');