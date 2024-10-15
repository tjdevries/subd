import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

document.addEventListener('DOMContentLoaded', () => {
  console.log('Page loaded, setting up animations and charts.');

  // THREE.js setup for a 3D scene featuring an owl and a raven
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
  const renderer = new THREE.WebGLRenderer();
  renderer.setSize(window.innerWidth, window.innerHeight);
  document.body.appendChild(renderer.domElement);

  console.log('Three.js renderer set up.');

  // Add lighting
  const darkNight = new THREE.AmbientLight(0x404040); // soft white light
  const moonLight = new THREE.DirectionalLight(0xffffff, 0.5);
  darkNight.position.set(10, 10, 10).normalize();
  scene.add(darkNight);
  scene.add(moonLight);

  let owl, raven;
  // Owl
  owl = new THREE.Mesh(
    new THREE.ConeGeometry(5, 20, 32),
    new THREE.MeshBasicMaterial({color: 0xffff00})
  );
  owl.position.x = -10;
  scene.add(owl);

  // Raven
  raven = new THREE.Mesh(
    new THREE.BoxGeometry(),
    new THREE.MeshBasicMaterial({color: 0x000000})
  );
  raven.position.x = 10;
  scene.add(raven);

  console.log('Owl and Raven meshes added to scene.');

  camera.position.z = 50;

  function animate() {
    requestAnimationFrame(animate);

    // Animating the owl and raven movement akin to their nocturnal brawl
    owl.rotation.x += 0.01;
    owl.rotation.y += 0.01;

    raven.rotation.y += 0.02;
    raven.rotation.z += 0.02;

    renderer.render(scene, camera);
  }

  animate();

  console.log('Animation loop started.');

  // D3.js for dynamic background animations
  const svg = D3.select('body').append('svg').attr('width', window.innerWidth).attr('height', window.innerHeight);
  svg.append('text').text('Feathered Fury').attr('x', 100).attr('y', 100).style('font-size', '40px').style('fill', 'white');

  console.log('D3.js text element added.');

  // Mermaid.js for visualizing fight sequences
  mermaid.initialize({startOnLoad:true});
  const graphDefinition = `
    graph TD;
      A(Owl Scouts) --> B{Moves}
      B -->|Soft Wing| C(Owl Wisdom)
      B -->|Sharp Claw| D(Raven Dive)
      C --> E[Victory]
      D --> E
  `;
  console.log('Mermaid.js chart initialized.');

  mermaid.render('mermaid', graphDefinition, (svgCode) => {
    document.body.innerHTML += svgCode;
    console.log('Mermaid graph rendered.');
  });

  console.log('All scripts executed.');
});