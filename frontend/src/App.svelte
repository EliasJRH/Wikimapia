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
  <button onclick={findShortestPath}>Go</button>
  {#if loading}
    <div class="loading-div">
      <h1 transition:fly={{ duration: 500 }}>
        <span>L</span>
        <span>O</span>
        <span>A</span>
        <span>D</span>
        <span>I</span>
        <span>N</span>
        <span>G</span>
      </h1>
    </div>
  {/if}
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

  .loading-div {
    position: relative;
    color: black
  }

  /* Copied from https://www.youtube.com/watch?v=eHJoKjMbKt4 */
  .loading-div h1 span {
    display: inline-block;
    animation: bounce 2s ease infinite;
  }

  .loading-div h1 span:nth-child(2) {
    animation-delay: 0.2s;
  }
  .loading-div h1 span:nth-child(3) {
    animation-delay: 0.4s;
  }
  .loading-div h1 span:nth-child(4) {
    animation-delay: 0.6s;
  }
  .loading-div h1 span:nth-child(5) {
    animation-delay: 0.8s;
  }
  .loading-div h1 span:nth-child(6) {
    animation-delay: 1s;
  }
  .loading-div h1 span:nth-child(7) {
    animation-delay: 1.2s;
  }

  @keyframes bounce {
    0%, 100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-20px);
    }
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
