document.addEventListener('DOMContentLoaded', () => {
  // Using three.js to create a fun 3D effect on the header
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

  function animate() {
    requestAnimationFrame(animate);

    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;

    renderer.render(scene, camera);
  }

  animate();

  // Adding some interactive animations for links using phaser.js
  const config = {
    type: Phaser.AUTO,
    width: 800,
    height: 600,
    scene: {
      preload: preload,
      create: create
    }
  };

  const game = new Phaser.Game(config);

  function preload() {
    this.load.setBaseURL('http://labs.phaser.io');
    this.load.image('logo', 'assets/sprites/phaser3-logo.png');
  }

  function create() {
    const logo = this.add.image(400, 70, 'logo');

    this.tweens.add({
      targets: logo,
      y: 500,
      duration: 2000,
      ease: 'Power2',
      yoyo: true,
      loop: -1
    });
  }

  // Text coloring animation using D3.js
  d3.selectAll('.title').transition().duration(2000)
    .style('color', 'magenta')
    .style('font-size', '2em');

  d3.selectAll('.nav-link').on('mouseover', function() {
    d3.select(this).style('color', 'orange');
  }).on('mouseout', function() {
    d3.select(this).style('color', 'black');
  });

});