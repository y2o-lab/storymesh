import { LitElement, html } from 'lit';
import { customElement } from 'lit/decorators.js';
@customElement('sample-greeting')
export class Greeting extends LitElement {
  render() { return html`<p>Lit greeting</p>`; }
}
