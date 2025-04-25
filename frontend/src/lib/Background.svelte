<script lang="ts">
  
</script>

<div class="background">
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
  <span></span>
</div>

<style>
  @keyframes move {
      100% {
          transform: translate3d(0, 0, 1px) rotate(360deg);
      }
  }

  .background {
      position: fixed;
      width: 100vw;
      height: 100vh;
      top: 0;
      left: 0;
      background: #ffffff;
      overflow: hidden;
  }

  .background span {
      width: 8vmin;
      height: 8vmin;
      border-radius: 8vmin;
      backface-visibility: hidden;
      position: absolute;
      animation: move;
      animation-duration: 31;
      animation-timing-function: linear;
      animation-iteration-count: infinite;
  }


  .background span:nth-child(0) {
      color: #0b0080;
      top: 11%;
      left: 50%;
      animation-duration: 149s;
      animation-delay: -106s;
      transform-origin: -18vw 25vh;
      box-shadow: -16vmin 0 2.496793942426474vmin currentColor;
  }
  .background span:nth-child(1) {
      color: #f8f9fa;
      top: 42%;
      left: 14%;
      animation-duration: 107s;
      animation-delay: -9s;
      transform-origin: 6vw -12vh;
      box-shadow: -16vmin 0 2.563696643864341vmin currentColor;
  }
  .background span:nth-child(2) {
      color: #a2a9b1;
      top: 56%;
      left: 28%;
      animation-duration: 51s;
      animation-delay: -159s;
      transform-origin: 20vw -10vh;
      box-shadow: -16vmin 0 2.4391997530536655vmin currentColor;
  }
  .background span:nth-child(3) {
      color: #a2a9b1;
      top: 81%;
      left: 27%;
      animation-duration: 189s;
      animation-delay: -86s;
      transform-origin: 1vw -19vh;
      box-shadow: 16vmin 0 2.189873898381272vmin currentColor;
  }
  .background span:nth-child(4) {
      color: #ffffff;
      top: 9%;
      left: 38%;
      animation-duration: 186s;
      animation-delay: -120s;
      transform-origin: -13vw -17vh;
      box-shadow: -16vmin 0 2.758881794457716vmin currentColor;
  }
  .background span:nth-child(5) {
      color: #0645ad;
      top: 34%;
      left: 62%;
      animation-duration: 9s;
      animation-delay: -36s;
      transform-origin: -19vw 9vh;
      box-shadow: 16vmin 0 2.291472159904337vmin currentColor;
  }
  .background span:nth-child(6) {
      color: #a2a9b1;
      top: 31%;
      left: 9%;
      animation-duration: 108s;
      animation-delay: -117s;
      transform-origin: 7vw 18vh;
      box-shadow: -16vmin 0 2.4894253029469477vmin currentColor;
  }
  .background span:nth-child(7) {
      color: #ffffff;
      top: 33%;
      left: 9%;
      animation-duration: 192s;
      animation-delay: -184s;
      transform-origin: 17vw -13vh;
      box-shadow: 16vmin 0 2.2560790352303552vmin currentColor;
  }
  .background span:nth-child(8) {
      color: #ffffff;
      top: 25%;
      left: 43%;
      animation-duration: 75s;
      animation-delay: -127s;
      transform-origin: 25vw -8vh;
      box-shadow: -16vmin 0 2.9108324121371965vmin currentColor;
  }
  .background span:nth-child(9) {
      color: #0645ad;
      top: 2%;
      left: 82%;
      animation-duration: 158s;
      animation-delay: -41s;
      transform-origin: 24vw -19vh;
      box-shadow: -16vmin 0 2.5508989754154587vmin currentColor;
  }
  .background span:nth-child(10) {
      color: #f8f9fa;
      top: 54%;
      left: 43%;
      animation-duration: 52s;
      animation-delay: -113s;
      transform-origin: -15vw -7vh;
      box-shadow: -16vmin 0 2.483753526364912vmin currentColor;
  }
  .background span:nth-child(11) {
      color: #ffffff;
      top: 39%;
      left: 86%;
      animation-duration: 88s;
      animation-delay: -58s;
      transform-origin: 25vw 12vh;
      box-shadow: -16vmin 0 2.134365764417892vmin currentColor;
  }
  .background span:nth-child(12) {
      color: #f8f9fa;
      top: 4%;
      left: 51%;
      animation-duration: 48s;
      animation-delay: -91s;
      transform-origin: 19vw -18vh;
      box-shadow: 16vmin 0 2.6803595933953828vmin currentColor;
  }
  .background span:nth-child(13) {
      color: #0b0080;
      top: 79%;
      left: 82%;
      animation-duration: 106s;
      animation-delay: -153s;
      transform-origin: -9vw -19vh;
      box-shadow: 16vmin 0 2.7611014556893845vmin currentColor;
  }
  .background span:nth-child(14) {
      color: #ffffff;
      top: 29%;
      left: 6%;
      animation-duration: 172s;
      animation-delay: -39s;
      transform-origin: -3vw 22vh;
      box-shadow: 16vmin 0 2.233650803536107vmin currentColor;
  }
  .background span:nth-child(15) {
      color: #0645ad;
      top: 66%;
      left: 32%;
      animation-duration: 54s;
      animation-delay: -93s;
      transform-origin: -20vw 13vh;
      box-shadow: -16vmin 0 2.1986624454937376vmin currentColor;
  }
  .background span:nth-child(16) {
      color: #ffffff;
      top: 87%;
      left: 66%;
      animation-duration: 92s;
      animation-delay: -74s;
      transform-origin: -14vw 6vh;
      box-shadow: 16vmin 0 2.1569844039274852vmin currentColor;
  }
  .background span:nth-child(17) {
      color: #f8f9fa;
      top: 43%;
      left: 74%;
      animation-duration: 162s;
      animation-delay: -188s;
      transform-origin: 21vw -14vh;
      box-shadow: -16vmin 0 2.7226709929663055vmin currentColor;
  }
  .background span:nth-child(18) {
      color: #ffffff;
      top: 100%;
      left: 26%;
      animation-duration: 179s;
      animation-delay: -63s;
      transform-origin: -10vw -24vh;
      box-shadow: 16vmin 0 2.92926568497523vmin currentColor;
  }
  .background span:nth-child(19) {
      color: #ffffff;
      top: 78%;
      left: 75%;
      animation-duration: 36s;
      animation-delay: -117s;
      transform-origin: 1vw 18vh;
      box-shadow: -16vmin 0 2.0463768409711927vmin currentColor;
  }
  .background span:nth-child(20) {
      color: #0645ad;
      top: 17%;
      left: 95%;
      animation-duration: 183s;
      animation-delay: -126s;
      transform-origin: -3vw -18vh;
      box-shadow: -16vmin 0 2.5596236895830016vmin currentColor;
  }
  .background span:nth-child(21) {
      color: #f8f9fa;
      top: 99%;
      left: 97%;
      animation-duration: 177s;
      animation-delay: -169s;
      transform-origin: -15vw -17vh;
      box-shadow: -16vmin 0 2.3143470858904927vmin currentColor;
  }
  .background span:nth-child(22) {
      color: #a2a9b1;
      top: 82%;
      left: 8%;
      animation-duration: 98s;
      animation-delay: -64s;
      transform-origin: 22vw 21vh;
      box-shadow: 16vmin 0 2.0495085385371286vmin currentColor;
  }
  .background span:nth-child(23) {
      color: #a2a9b1;
      top: 38%;
      left: 56%;
      animation-duration: 76s;
      animation-delay: -184s;
      transform-origin: 2vw 11vh;
      box-shadow: 16vmin 0 2.7251784172243374vmin currentColor;
  }
  .background span:nth-child(24) {
      color: #0b0080;
      top: 9%;
      left: 78%;
      animation-duration: 25s;
      animation-delay: -76s;
      transform-origin: -22vw 9vh;
      box-shadow: 16vmin 0 2.5120452505093653vmin currentColor;
  }
</style>