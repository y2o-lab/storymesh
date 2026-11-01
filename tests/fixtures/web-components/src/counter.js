export class Counter extends HTMLElement {
  connectedCallback() { this.textContent = 'Native element'; }
}
customElements.define('sample-counter', Counter);
