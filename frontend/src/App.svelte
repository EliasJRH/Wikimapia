<script lang="ts">
  import { fly } from 'svelte/transition';

  import arrowRight from './assets/arrow-right.svg'
  import Background from './lib/Background.svelte';
  import DisplayText from './lib/DisplayText.svelte';
  import DropdownSelect from './lib/DropdownSelect.svelte';
  import Footer from './lib/Footer.svelte';
  import Header from './lib/Header.svelte';

  let searchUrl = "http://localhost:8080/path?"

  let startingArticle = $state("")
  let endingArticle = $state("")
  let foundPath: string[] = $state([])
  let loading = $state(false)

  const findShortestPath = () => {
    console.log(`${startingArticle} -> ${endingArticle}`)
    let params = new URLSearchParams({
        startpage: startingArticle,
        endpage: endingArticle
      })
    loading = true
    fetch(searchUrl + params.toString()).then(res => res.json()).then(data => {
      console.log(data)
      loading = false
    })
  }
</script>

<main>
  <Background/>
  <Header/>
  <DisplayText/>
  <div class="inputs-holder">
    <DropdownSelect bind:articleName={startingArticle} placeholder_text = "Starting article"/>
    <img src={arrowRight} width="50px" height="auto" alt="">
    <DropdownSelect bind:articleName={endingArticle} placeholder_text = "Ending article"/>
  </div>
  <button onclick={() => console.log("Gone")}>Go</button>
  <!-- <Footer/> -->
   
</main>

<style>
  .inputs-holder{
    display: flex;
    flex-direction: row;
    justify-content: space-evenly;
    margin-bottom: 5vh;
  }

  .inputs-holder img{
    position: absolute; 
    transform:scale(2)
  }

  @media (max-width: 768px) {
    .inputs-holder {
      flex-direction: column;
      align-items: center;
      justify-content: space-evenly;
      height: 30vh;
    }

    .inputs-holder img{
      transform: rotate(90deg);
    }
  }
</style>
