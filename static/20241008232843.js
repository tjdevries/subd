document.addEventListener('DOMContentLoaded', function () {
  // Function to animate the header text
  function animateHeader() {
    const header = document.querySelector('.header');
    header.style.transition = 'transform 0.3s ease';
    header.style.transform = 'scale(1.1)';

    setTimeout(() => {
      header.style.transform = 'scale(1)';
    }, 300);
  }

  // Function to animate nav links with hover effect
  function animateNavLinks() {
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
      link.addEventListener('mouseover', () => {
        link.style.transition = 'color 0.3s ease';
        link.style.color = '#FF4081';
      });

      link.addEventListener('mouseout', () => {
        link.style.color = '';
      });
    });
  }

  // Function to animate song items on hover
  function animateSongs() {
    const songs = document.querySelectorAll('.song, .song-container');
    songs.forEach(song => {
      song.addEventListener('mouseover', () => {
        song.style.transition = 'box-shadow 0.3s ease, transform 0.3s ease';
        song.style.boxShadow = '0 2px 10px rgba(0, 0, 0, 0.15)';
        song.style.transform = 'translateY(-5px)';
      });

      song.addEventListener('mouseout', () => {
        song.style.boxShadow = '';
        song.style.transform = '';
      });
    });
  }

  // Function to load dynamic backgrounds for sections
  function loadDynamicBackgrounds() {
    const sections = document.querySelectorAll('section');
    sections.forEach(section => {
      section.style.transition = 'background-color 0.5s ease';
      section.style.backgroundColor = '#f9f9f9';
    });
  }

  // Call animation functions
  animateHeader();
  animateNavLinks();
  animateSongs();
  loadDynamicBackgrounds();

  // Apply animations periodically
  setInterval(animateHeader, 4000);
});
