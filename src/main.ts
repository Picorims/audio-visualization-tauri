// audio-visualization-tauri is an audio visualization experiment
// Copyright (C) 2023  Charly Schmidt alias Picorims<picorims.contact@gmail.com>

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

import "./styles.css";
import App from "./App.svelte";

const app = new App({
  target: document.getElementById("app") as Element,
});

export default app;
