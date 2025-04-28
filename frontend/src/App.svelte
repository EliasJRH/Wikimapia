<script lang="ts">
  import { blur, fly } from 'svelte/transition';

  import arrowRight from './assets/arrow-right.svg'
  import Background from './lib/Background.svelte';
  import DisplayText from './lib/DisplayText.svelte';
  import DropdownSelect from './lib/DropdownSelect.svelte';
  import Card from './lib/Card.svelte';
  import Footer from './lib/Footer.svelte';
  import Header from './lib/Header.svelte';

  let searchUrl = "http://localhost:8080/path?"

  // type ArticleInfo = {
  //   title: string;
  //   description: string;
  // };

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
    fetch(searchUrl + params.toString())
      .then(res => res.json())
      .then(data => {
        console.log(data)
        // foundPath = data.map((item: any) => ({
        //   title: item.title,
        //   description: item.description
        // }))
        foundPath = data.path
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
        <span style="--delay: 0s;">L</span>
        <span style="--delay: 0.2s;">O</span>
        <span style="--delay: 0.4s;">A</span>
        <span style="--delay: 0.6s;">D</span>
        <span style="--delay: 0.8s;">I</span>
        <span style="--delay: 1s;">N</span>
        <span style="--delay: 1.2s;">G</span>
      </h1>
    </div>
  {:else if foundPath.length > 0}
    <div transition:fly={{ duration: 500 }} class="path-div">
      {#each foundPath as page, index}
        <span transition:blur={{ delay: (index * 0.4) * 10000 }} style:transform={index % 2 == 0 ? 'translateY(3vh)' : 'translateY(-3vh)'}><Card cardArticleName={page} cardArticleDesc={page}/></span>
        {#if index + 1 !== foundPath.length} <img transition:blur={{ delay: ((index * 0.4) + 0.1) * 10000 }} style:transform='scale(1.8) {index % 2 == 0 ? 'rotate(-30deg)' : 'rotate(30deg)'}' src={arrowRight} width="50px" height="auto" alt=""> {/if}
      {/each}
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

  .loading-div h1 span {
    animation-delay: var(--delay, 0s);
  }

  @keyframes bounce {
    0%, 100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-20px);
    }
  }

  .path-div{
    align-items: center;
    display:flex;
    flex-direction: row;
    justify-content: center;
    margin-top: 3vh;
  }

  .path-div img {
    position: relative;
    transform: scale(1.8);
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
