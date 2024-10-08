// Animate header text with a bounce effect using GSAP
const tl = gsap.timeline({repeat: -1, yoyo: true});

 tl.from('.header', {duration: 2, y: -50, ease: 'bounce', color: '#ff5733'})
   .from('.sub-header', {duration: 1.5, y: -30, opacity: 0, ease: 'bounce.out', stagger: 0.5});

// Animate nav links with a highlight on hover
document.querySelectorAll('.nav-link').forEach(link => {
   link.addEventListener('mouseenter', (e) => {
     gsap.to(e.target, {duration: 0.5, scale: 1.1, color: '#33ff57'});
   });
   link.addEventListener('mouseleave', (e) => {
     gsap.to(e.target, {duration: 0.3, scale: 1, color: '#000'});
   });
});

// Stagger animation for songs in playlist
gsap.from('.unplayed_song', {
  duration: 1.5,
  x: -100,
  opacity: 0,
  ease: 'back',
  stagger: 0.2
});

// Animate current song info
const currentSongTl = gsap.timeline();
currentSongTl.from('.current-song-info h3', {duration: 1.5, x: -100, opacity: 0, ease: 'elastic.out(1, 0.3)'})
  .from('.current-song-info .title', {duration: 1, x: 100, opacity: 0, ease: 'back.out'})
  .from('.current-song-info .tags', {duration: 1, x: -50, opacity: 0, ease: 'slow', delay: 0.5})
  .from('.current-song-info .creator', {duration: 1, x: 50, opacity: 0, ease: 'slow'});

// Animate images with hover effects
document.querySelectorAll('.ai_song_image img').forEach(img => {
    img.addEventListener('mouseenter', () => {
        gsap.to(img, {duration: 0.7, scale: 1.2});
    });
    img.addEventListener('mouseleave', () => {
        gsap.to(img, {duration: 0.5, scale: 1});
    });
});

// Animate video hover
const videos = document.querySelectorAll('.video');
videos.forEach(video => {
    video.addEventListener('mouseover', () => {
        gsap.to(video, {duration: 0.5, scale: 1.05});
    });
    video.addEventListener('mouseout', () => {
        gsap.to(video, {duration: 0.5, scale: 1});
    });
});