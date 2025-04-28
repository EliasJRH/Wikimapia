<script lang="ts">
  let { cardArticleName, cardArticleDesc } = $props();
</script>

<div class="path-card">
  <h2>{cardArticleName}</h2>
  <h4><i>{cardArticleDesc}</i></h4>
</div>

<style>
  .path-card{
    padding: 10px 15px;
    position: relative;
    width:11vw;
    height: 31vh;
    background-color: #6d6a6ae0;
    border-radius: 5px;
    box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2);
    transition: 0.3s;
    text-align: left;
    margin: 0 3vw;
  }
</style>