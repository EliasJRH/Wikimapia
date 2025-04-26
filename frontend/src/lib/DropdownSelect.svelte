<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly, slide } from 'svelte/transition';
  
  const search_titles_url = "https://en.wikipedia.org/w/rest.php/v1/search/title?";
  let props = $props();
	let message = $state("");
  let selectedIndex = $state(-1);
  let open = $state(false)
  let possible_articles: string[] = $state([])
  
  $effect(() => {
    if (message.length > 0){
      let params = new URLSearchParams({
        q: message,
        limit: "10"
      })
      fetch(search_titles_url + params.toString())
        .then(res => res.json())
        .then(data => {
          possible_articles = data?.pages.map((page: any) => page?.title) ?? [];
          selectedIndex = -1;
        })
        .catch(err => console.error("Error fetching articles:", err));
      const computeTotalHeight = () => {
        const listItems = document.getElementsByClassName('listItem');
        const totalHeight = Array.from(listItems).reduce((sum, item) => sum + (item as HTMLElement).offsetHeight, 0);
        const dd = document.querySelector(".dropdown")
        console.log(dd)
        document.querySelector(".dropdown")?.setAttribute("height", `${totalHeight}px`)
      };
      computeTotalHeight();
    }
  })

  const showList = () => {
    open = true;
  }

  const selectArticle = (article: any) => {
    message = article;
    open = false;
  };
</script>

<div style="width: 14vw;">
  <input 
    bind:value={message} 
    oninput={showList} 
    placeholder={props.placeholder_text} 
    onkeydown={(e) => {
      if (e.key === 'ArrowDown' && possible_articles.length > 0) {
        const firstItem = document.querySelector('.dropdown li');
        if (firstItem) (firstItem as HTMLElement).focus();
      }
    }} 
  />
  {#if open}
  <ul transition:slide={{ duration: 500 }} class="dropdown">
    {#each possible_articles as article, index}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <li
          class="{index === selectedIndex ? 'selected' : ''} listItem"
          onclick={() => selectArticle(article)}
          onkeydown={(e) => {
            if (e.key === 'ArrowDown') selectedIndex = Math.min(selectedIndex + 1, possible_articles.length - 1);
            if (e.key === 'ArrowUp') selectedIndex = Math.max(selectedIndex - 1, 0);
            if (e.key === 'Enter' && selectedIndex > -1) selectArticle(possible_articles[selectedIndex]);
          }}
          onfocus={() => {
            open = true
            selectedIndex = index}
          }
          onmouseenter={() => selectedIndex = index}
          tabindex=0
          >
          {article}
        </li>
        {/each}
    </ul>
  {/if}
</div>
  

<style>
  input {
    position: relative;
    background-color: #a4a1a166;
    border-radius: 10px;
    color: black;
    font-size: 1.2rem; /* Increase font size */
    padding: 10px 15px; /* Add padding for a larger input */
    border: none;
    height: auto;
    width: 14vw
  }
  input:focus {
    outline: none;
    box-shadow: 0px 4px 6px rgba(50, 115, 227, 0.5);
    transition: box-shadow 0.2s ease-in-out;
  }
  .dropdown {
    color: black;
    position: relative;
    background-color: white;
    border: 1px solid #ccc;
    border-radius: 5px;
    overflow-y: none;
    scroll-behavior: smooth;
    text-align: left;
    height: auto;
    z-index: 1000;  
    box-sizing: border-box;
  }
  .dropdown li {
    padding: 10px;
    height: fit-content;
  }

  ul{
    display: block;
    list-style-type: none;
    margin: 0;
    padding: 0;
  }
  .dropdown li:hover,
  .dropdown li.selected {
    background-color: #007bff;
    color: white;
  }

</style>