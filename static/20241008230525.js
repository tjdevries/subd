document.addEventListener("DOMContentLoaded", function() {
  // Animate header text
  const header = document.querySelector('.header');
  header.style.transition = "transform 0.5s ease-in-out";
  header.addEventListener('mouseover', () => {
      header.style.transform = 'scale(1.1)';
  });
  header.addEventListener('mouseout', () => {
      header.style.transform = 'scale(1)';
  });

  // Animate nav links on hover
  const navLinks = document.querySelectorAll('.nav-link');
  navLinks.forEach(link => {
    link.style.transition = "color 0.3s";
    link.addEventListener('mouseover', function() {
      this.style.color = 'orange';
    });
    link.addEventListener('mouseout', function() {
      this.style.color = "";
    });
  });

  // Pulsating effect on song titles
  const songTitles = document.querySelectorAll('.title');
  songTitles.forEach(title => {
    title.style.transition = "color 0.6s ease, font-size 0.6s ease";
    title.addEventListener('mouseover', function() {
      this.style.color = '#ff8822';
      this.style.fontSize = '1.2em';
    });
    title.addEventListener('mouseout', function() {
      this.style.color = "";
      this.style.fontSize = "";
    });
  });

  // Animate images
  const songImages = document.querySelectorAll('.ai_song_image img');
  songImages.forEach(img => {
    img.style.transition = "transform 0.2s";
    img.addEventListener('mouseover', () => {
      img.style.transform = 'scale(1.05)';
    });
    img.addEventListener('mouseout', () => {
      img.style.transform = 'scale(1)';
    });
  });

  // Bouncing effect on vote text
  const votesText = document.querySelectorAll('.image_voting');
  votesText.forEach(vote => {
    vote.style.transition = "transform 0.3s";
    vote.addEventListener('mouseover', () => {
      vote.style.transform = 'translateY(-5px)';
    });
    vote.addEventListener('mouseout', () => {
      vote.style.transform = 'translateY(0)';
    });
  });

  // Add fade in effect on sections
  const sections = document.querySelectorAll('section');
  sections.forEach(section => {
    section.style.opacity = '0';
    section.style.transition = 'opacity 0.5s';
    window.addEventListener('scroll', () => {
      const sectionTop = section.getBoundingClientRect().top;
      if (sectionTop < window.innerHeight - 150) {
        section.style.opacity = '1';
      }
    });
  });
});