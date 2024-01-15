//const fileInput = document.getElementById("file-input");
//const speechBubble = document.querySelector(".speech-bubble");


////-
//fileInput.addEventListener("change", (event) => {
//  let file = event.target.files[0];
  
//  if (file) {
//    let reader = new FileReader();

//    reader.onload = (e) => {
//      let content = e.target.result;
//      showTextInSpeechBubble(content);
//    };

//    reader.readAsText(file);
//  }
//});

//function showTextInSpeechBubble(text) {
//  speechBubble.textContent = text;
//  speechBubble.style.display = "block";
//}


function createSpeechBubble() {
    const message = document.getElementById('message').value;

    if (message.trim() === '' || !message) {
        return;
    }

    const container = document.getElementById('container');
    const speechBubble = document.createElement('div');
    
    speechBubble.className = 'speech-bubble';
    speechBubble.innerHTML = message;

    container.appendChild(speechBubble);
    document.getElementById('message').value = '';
}
