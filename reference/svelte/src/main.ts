import { mount } from "svelte";
import Environment from "./stories/Environment.svelte";
import "../../../dist/fonts/typography.css";

const target = document.getElementById("app");
if (!target) throw new Error("Fjord reference requires an #app element.");

mount(Environment, { target });
