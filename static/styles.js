// File: /static/styles.js

import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

// Initialize mermaid
mermaid.initialize({ startOnLoad: true });
console.log('Mermaid initialized');

// Function for animated background using Three.js
function initThreeJSBackground() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();

    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    const geometry = new THREE.SphereGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0xfffff, wireframe: true });
    const sphere = new THREE.Mesh(geometry, material);
    scene.add(sphere);

    camera.position.z = 5;

    function animate() {
        requestAnimationFrame(animate);

        sphere.rotation.x += 0.01;
        sphere.rotation.y += 0.01;

        renderer.render(scene, camera);
    }

    animate();
    console.log('Three.js background animation initialized');
}

initThreeJSBackground();

// Function for D3 chart
function initD3Chart() {
    const data = [30, 86, 168, 281, 303, 365];
    const width = 420;
    const barHeight = 20;

    const chart = D3.select("body")
        .append("svg")
        .attr("width", width)
        .attr("height", barHeight * data.length);

    const bar = chart.selectAll("g")
        .data(data)
        .enter().append("g")
        .attr("transform", (d, i) => "translate(0," + i * barHeight + ")");

    bar.append("rect")
        .attr("width", d => d)
        .attr("height", barHeight - 1);

    bar.append("text")
        .attr("x", d => d - 3)
        .attr("y", barHeight / 2)
        .attr("dy", ".35em")
        .text(d => d);

    console.log('D3 chart created');
}

initD3Chart();

// Function for animated Mermaid charts
function initMermaid() {
    const graphDefinition = `graph TD;
        A[Start] --> B{Is it?};
        B -- Yes --> C[OK];
        C --> D[Rethink];
        D --> A;
        B -- No --> E[Lament];
        E --> F[End];
    `;

    const mermaidDiv = document.createElement('div');
    mermaidDiv.classList.add('mermaid');
    mermaidDiv.innerHTML = graphDefinition;
    document.body.appendChild(mermaidDiv);

    mermaid.init(undefined, mermaidDiv);
    console.log('Mermaid chart initialized');
}

initMermaid();

// Export the JavaScript as a module
console.log('JavaScript animation and charts setup completed');