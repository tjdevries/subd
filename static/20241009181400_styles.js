import * as THREE from 'three';
import * as d3 from 'd3';
import mermaid from 'mermaid';

console.log('Starting to create a magical musical atmosphere!');

// Initialize Three.js scene
document.addEventListener('DOMContentLoaded', function() {
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

    console.log('3D scene initialized with a rotating cube.');

    function animate() {
        requestAnimationFrame(animate);
        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;
        renderer.render(scene, camera);
    }
    animate();

    window.addEventListener('resize', function() {
        const width = window.innerWidth;
        const height = window.innerHeight;
        renderer.setSize(width, height);
        camera.aspect = width / height;
        camera.updateProjectionMatrix();
    });

    console.log('Animation and resize handlers set up.');
});

// Initialize D3.js chart
document.addEventListener('DOMContentLoaded', function() {
    const svg = d3.select('body').append('svg')
        .attr('width', 500)
        .attr('height', 300);

    const dataset = [10, 20, 30, 40, 50];
    svg.selectAll('circle')
        .data(dataset)
        .enter()
        .append('circle')
        .attr('cx', (d, i) => i * 100 + 50)
        .attr('cy', 150)
        .attr('r', (d) => d)
        .attr('fill', 'purple');

    console.log('D3.js chart created displaying circles.');
});

// Initialize Mermaid.js
document.addEventListener('DOMContentLoaded', function() {
    mermaid.initialize({ startOnLoad: true });

    const graph = `graph TD;
        A[AI Generates] --> B{Songs}
        B --> C[Love Songs]
        B --> D[Heartbreak Songs]
        B --> E[Adventure Songs]
        C --> |Vote| F[Hit Chart]
        D --> |Vote| F
        E --> |Vote| F`;

    const mermaidContainer = document.createElement('div');
    mermaidContainer.classList.add('mermaid');
    mermaidContainer.innerHTML = graph;
    document.body.appendChild(mermaidContainer);

    mermaid.init(undefined, mermaidContainer);

    console.log('Mermaid chart initialized showing AI song types and charts.');
});

console.log('JavaScript loaded and running.');