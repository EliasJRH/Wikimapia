<script lang="ts">
  import arrowRighy from './assets/arrow-right.svg'

  import Background from './lib/Background.svelte';
  import DisplayText from './lib/DisplayText.svelte';
  import DropdownSelect from './lib/DropdownSelect.svelte';
  import Footer from './lib/Footer.svelte';
  import Header from './lib/Header.svelte';
</script>

<main>
  <Background/>
  <Header/>
  <DisplayText/>
  <div class="input-holder">
    <DropdownSelect placeholder_text = "Starting page"/>
    <img src={arrowRighy} width="50px" height="auto" alt="" style="position: absolute;">
    <DropdownSelect placeholder_text = "Ending page"/>
  </div>
  <!-- <Footer/> -->
   
</main>

<style>
  .input-holder{
    display: flex;
    flex-direction: row;
    justify-content: space-evenly;
  }

  .input-holder img{
    position: relative;
  }

  @media (max-width: 768px) {
    .input-holder {
      flex-direction: column;
      align-items: center;
      justify-content: space-evenly;
    }
  }
</style>
