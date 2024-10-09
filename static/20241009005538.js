// This script utilizes three.js to add 3D elements and animations to the page. 
// The concept is to create a playful representation of 'modal madness' visually and with fun interactions.

// Importing required THREE library
import * as THREE from 'three';

// Scene setup
overlayScene();
function overlayScene() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer({ alpha: true });
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    // Ambient and Point Light
    const ambientLight = new THREE.AmbientLight(0x404040);
    scene.add(ambientLight);
    
    const pointLight = new THREE.PointLight(0xffffff);
    pointLight.position.set(50, 50, 50);
    scene.add(pointLight);

    // Modal cube representing the chaotic pop-ups
    const geometry = new THREE.BoxGeometry(1, 1, 1);
    const material = new THREE.MeshStandardMaterial({ color: 0x00ff00, wireframe: true });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);
    cube.position.set(Math.random() - 0.5, Math.random() - 0.5, Math.random() - 0.5).multiplyScalar(10);
    
    // Animation of modal cubes
    function animateCube() {
        requestAnimationFrame(animateCube);
        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;
        cube.position.x += (Math.random() - 0.5) * 0.1;
        cube.position.y += (Math.random() - 0.5) * 0.1;
        renderer.render(scene, camera);
    }
    animateCube();

    camera.position.z = 5;
}