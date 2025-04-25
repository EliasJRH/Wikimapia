<script lang="ts">
  import TypeIt from "typeit";
  let randomTopics = [
    "Rubik's Cube", 
    "Photosynthesis", 
    "Napoleon Bonaparte", 
    "Bitcoin", 
    "Loch Ness Monster", 
    "Big Bang Theory", 
    "Taj Mahal", 
    "Leonardo da Vinci", 
    "Theory of Relativity", 
    "Wright Brothers", 
    "Eiffel Tower", 
    "Mona Lisa", 
    "Galileo Galilei", 
    "Antarctica", 
    "Great Barrier Reef", 
    "Isaac Newton", 
    "Amazon Rainforest", 
    "Albert Einstein", 
    "Statue of Liberty", 
    "Golden Gate Bridge", 
    "Pluto", 
    "Hubble Telescope", 
    "Turing Machine", 
    "CERN", 
    "Apollo 11", 
    "The Simpsons", 
    "Doppler Effect", 
    "Machu Picchu", 
    "Artificial Intelligence", 
    "Peanut Butter",
    "Quantum Computing",
    "Black Holes",
    "Great Wall of China",
    "Marie Curie",
    "Blockchain",
    "Pyramids of Giza",
    "String Theory",
    "Shakespeare",
    "Electric Cars",
    "Mount Everest"
  ]
  randomTopics = randomTopics.sort(() => Math.random() - 0.5);
  document.addEventListener("DOMContentLoaded", function () {
    new TypeIt("#test", {
      speed: 50,
      // strings: [`How do you get from ${randomTopics[0]} to ${randomTopics[1]}?`, `${randomTopics[2]} to ${randomTopics[3]}?`],
      lifeLike: true,
      breakLines: false,
      nextStringDelay: 1000,
    }).type(`How do you get from ${randomTopics[0]} to ${randomTopics[1]}?`)
      .pause(2000)
      .move(-1)
      .delete(randomTopics[1].length)
      .move(-4)
      .delete(randomTopics[0].length)
      .pause(500)
      .type(randomTopics[2])
      .move(4)
      .type(randomTopics[3])
      .pause(2000)
      .delete(randomTopics[3].length)
      .move(-4)
      .delete(randomTopics[2].length)
      .pause(500)
      .type(randomTopics[5])
      .move(4)
      .type(randomTopics[6])
      .pause(2000)
      .delete(randomTopics[6].length)
      .move(-4)
      .delete(randomTopics[5].length)
      .pause(500)
      .type(randomTopics[7])
      .move(4)
      .type(randomTopics[8])
      // .options({cursor: false, cursorChar: ""})
      .go()
  })
</script>

<div class="display-text">
  <h1>Connect the dots of curiosity.</h1>
  <h2 id="test"></h2>
</div>

<style>
  .display-text {
    color: black;
    position: relative;
    left: 0;
    margin: auto;
    display: flex;
    justify-content: center;
    align-items: center;
    text-align: center;
    min-height: 50vh;
    flex-direction: column;
  }

  .display-text h1 {
    font-size: 5rem;
    margin: 0;
  }

  @media (max-width: 768px) {
    .display-text h1 {
      font-size: 2rem;
      min-width: 80vw;
    }
  }
</style>