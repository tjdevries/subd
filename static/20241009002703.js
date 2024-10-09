document.addEventListener('DOMContentLoaded', () => {
  // Initialize Three.js scene
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
  const renderer = new THREE.WebGLRenderer();
  renderer.setSize(window.innerWidth, window.innerHeight);
  document.body.appendChild(renderer.domElement);

  const geometry = new THREE.CircleGeometry(5, 32);
  const material = new THREE.MeshBasicMaterial({ color: 0xffff00 });
  const circle = new THREE.Mesh(geometry, material);
  scene.add(circle);
  camera.position.z = 20;

  const animateCircle = () => {
    requestAnimationFrame(animateCircle);
    circle.rotation.x += 0.01;
    circle.rotation.y += 0.01;
    renderer.render(scene, camera);
  };

  animateCircle();

  // Animate elements of the page
  const navLinks = document.querySelectorAll('.nav-link');
  setInterval(() => {
    navLinks.forEach(link => {
      link.style.transition = 'color 0.5s ease-in-out';
      link.style.color = '#' + Math.floor(Math.random()*16777215).toString(16);
    });
  }, 1000);

  const headers = document.querySelectorAll('.header-container h2');
  headers.forEach(header => {
    header.style.position = 'absolute';
    header.style.animation = 'float 3s ease-in-out infinite';
  });

  const style = document.createElement('style');
  style.type = 'text/css';
  style.innerHTML = `
    @keyframes float {
        0% { transform: translatey(0px); }
        50% { transform: translatey(-20px); }
        100% { transform: translatey(0px); }
    }
  `;
  document.getElementsByTagName('head')[0].appendChild(style);

  // Adding pulse effect to all h1 elements
  const h1Elements = document.querySelectorAll('h1');
  h1Elements.forEach(h1 => {
    h1.style.animation = 'pulse 2s infinite';
  });

  style.innerHTML += `
    @keyframes pulse {
      0% { transform: scale(1); }
      50% { transform: scale(1.1); }
      100% { transform: scale(1); }
    }
  `;

});