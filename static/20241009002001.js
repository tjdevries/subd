// JavaScript code to animate your web page
// Using Three.js and GSAP for animations

// Import necessary libraries
import * as THREE from 'https://cdn.jsdelivr.net/npm/three@0.119/build/three.module.js';
import { OrbitControls } from 'https://cdn.jsdelivr.net/npm/three@0.119/examples/jsm/controls/OrbitControls.js';
import { gsap } from 'https://cdn.jsdelivr.net/npm/gsap@3.6.0/dist/gsap.min.js';

// Initialize Three.js scene
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Add orbit controls
const controls = new OrbitControls(camera, renderer.domElement);

// Add ambient light
const light = new THREE.AmbientLight(0x404040); // Soft white light
scene.add(light);

// Add a point light
const pointLight = new THREE.PointLight(0xffffff, 1);
pointLight.position.set(5, 5, 5);
scene.add(pointLight);

// Create a rotating cube
const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshStandardMaterial({ color: 0x0088ff });
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);

camera.position.z = 5;

// GSAP animation for cube rotation
function animateCube() {
  gsap.to(cube.rotation, { y: cube.rotation.y + Math.PI, duration: 2, ease: 'power1.inOut', onComplete: animateCube });
}

animateCube();

// Render the scene
function animate() {
  requestAnimationFrame(animate);
  controls.update();
  renderer.render(scene, camera);
}
animate();

// Fun animations for navigation links
const navLinks = document.querySelectorAll('.nav-link');
navLinks.forEach((link, i) => {
  gsap.to(link, { 
    duration: 2.5, 
    x: 20 + i * 10, 
    yoyo: true, 
    repeat: -1, 
    ease: 'elastic.out'
  });
});

// Enhance header appearance with bouncing effect
const headers = document.querySelectorAll('.header, .sub-header');
headers.forEach(header => {
  gsap.fromTo(header, { 
    scale: 0.8 
  }, { 
    duration: 1.5, 
    scale: 1, 
    ease: 'bounce.out',
    repeat: -1,
    yoyo: true
  });
});