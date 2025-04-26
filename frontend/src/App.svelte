<script lang="ts">
  import arrowRight from './assets/arrow-right.svg'
  import Background from './lib/Background.svelte';
  import DisplayText from './lib/DisplayText.svelte';
  import DropdownSelect from './lib/DropdownSelect.svelte';
  import Footer from './lib/Footer.svelte';
  import Header from './lib/Header.svelte';

  let foundPath: string[] = $state([])
</script>

<main>
  <Background/>
  <Header/>
  <DisplayText/>
  <div class="inputs-holder">
    <DropdownSelect placeholder_text = "Starting article"/>
    <img src={arrowRight} width="50px" height="auto" alt="">
    <DropdownSelect placeholder_text = "Ending article"/>
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
