<script context="module" lang="ts">
  export interface Recording {
    created: string;
    uid: number;
  }

  export function loglin(x: number, minfreq: number, maxfreq: number) {
    
      return (maxfreq - minfreq)*Math.log10(x/minfreq)/Math.log10(maxfreq/minfreq) + minfreq;

  }

  export function linlog(x: number, x1: number, x2: number) {
    
      return x1 * Math.pow(10, (x - x1)/(x2 - x1) * Math.log10(x2 / x1));

  }
</script>
