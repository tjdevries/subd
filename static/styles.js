// Import Three.js
import * as THREE from 'three';

// Import the Animation libaries
import { TweenMax } from 'gsap';

// Setup a basic WebGL renderer, scene, and camera
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Add a spinning cube
const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);

camera.position.z = 5;

const animate = function () {
  requestAnimationFrame(animate);

  cube.rotation.x += 0.01;
  cube.rotation.y += 0.01;

  renderer.render(scene, camera);
};

animate();

// Adding fun animations using TweenMax
function bounceElement(selector) {
  let element = document.querySelector(selector);
  TweenMax.to(element, 2, {y:60, ease:Bounce.easeOut, repeat:-1, yoyo:true});
}

// Select elements and animate them
bounceElement('.header-container h1');
bounceElement('.sub-header');
bounceElement('.nav-link');

// Function to create starry background
function createStarField() {
  for (let i = 0; i < 1000; i++) {
    const starGeometry = new THREE.SphereGeometry(0.1);
    const starMaterial = new THREE.MeshBasicMaterial({ color: 0xffffff });
    const starMesh = new THREE.Mesh(starGeometry, starMaterial);
    starMesh.position.set(
      Math.random() * 100 - 50,
      Math.random() * 100 - 50,
      Math.random() * 100 - 50
    );
    scene.add(starMesh);
  }
}

createStarField();

// Add event listeners to images for animations
document.querySelectorAll('.ai_song_image img').forEach(img => {
  img.addEventListener('mouseover', () => TweenMax.to(img, 0.5, { rotation: 360 }));
});

document.querySelectorAll('.ai_song_image img').forEach(img => {
  img.addEventListener('mouseout', () => TweenMax.to(img, 0.5, { rotation: 0 }));
});