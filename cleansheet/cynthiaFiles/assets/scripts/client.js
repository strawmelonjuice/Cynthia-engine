/*
Cynthia Client-side script.

This script is embedded into any page Cynthia serves, always just before the closing </html>.
*/
/** @typedef {object} cynthia
 * @property {string} version
 * @property {object} publicationdata
 * @property {string} publicationdata.id
 * @property {string} publicationdata.title
 * @property {string} publicationdata.desc
 * @property {string} publicationdata.category
 * @property {string[]} publicationdata.tags
 * @property {object} publicationdata.author
 * @property {string} publicationdata.author.name
 * @property {string} publicationdata.author.thumbnail
 * @property {string} publicationdata.author.link
 * @property {object} publicationdata.dates
 * @property {number} publicationdata.dates.altered
 * @property {number} publicationdata.dates.published
 * @property {string} publicationdata.thumbnail
 * @property {string} kind
 */

console.log("Cynthia client-side script loaded.");

setInterval(() => {
  const elements = document.getElementsByClassName("unparsedtimestamp");
  for (let i = elements.length - 1; i >= 0; i--) {
    const timestamp = Number.parseInt(elements[i].textContent);
    console.log("Parsing timestamp.");
    const jstimestamp = timestamp * 1000;
    const dateObject = new Date(jstimestamp);
    const data = dateObject.toLocaleString();
    elements[i].textContent = data.substring(0, data.length - 3);
    elements[i].classList.remove("unparsedtimestamp");
  }
}, 100);

function mobileorientation() {
  const csssays = getComputedStyle(document.body).getPropertyValue(
    "--screen-type-orientation",
  );
  if (csssays === "mobile") {
    return 1;
  }
  if (csssays === "landscape") {
    return 0;
  }
  console.error(
    `Could not determine 'mobilescreen()' from css value '${csssays}'.`,
  );
}

if (document.getElementById("pageinfosidebar") != null) {
  if (cynthia.kind === "post") {
    console.info("This is a post. Displaying pageinfosidebar.");

    function pageinfosidebar_rollup() {
      document.getElementById("pageinfosidebar").style.overflow = "hidden";
      document.getElementById("pageinfosidebar").style.width = "0";
      document.getElementById("pageinfosidebar").style.maxHeight = "310";
      document.getElementById("pageinfosidebartoggle").style.display = "";
      setTimeout(() => {
        document.getElementById("pageinfosidebartoggle").style.width = "40";
      }, 1700);
      setTimeout(() => {
        document.getElementById("pageinfosidebartoggle").style.padding = "8px";
      }, 1800);
    }
    document
      .getElementById("pageinfosidebar-rollup")
      .addEventListener("click", pageinfosidebar_rollup);
    function pageinfosidebar_rollout() {
      document.getElementById("pageinfosidebar").style.overflow = "";
      document.getElementById("pageinfosidebar").style.opacity = "100%";
      document.getElementById("pageinfosidebartoggle").style.overflow =
        "hidden";
      document.getElementById("pageinfosidebartoggle").style.width = "0";
      document.getElementById("pageinfosidebartoggle").style.padding = "0";
      setTimeout(() => {
        document.getElementById("pageinfosidebar").style.width = "";
      }, 1800);
      setTimeout(() => {
        document.getElementById("pageinfosidebar").style.height = "";
      }, 1900);
    }

    if (mobileorientation() || window.innerHeight < 350) {
      setTimeout(() => {
        pageinfosidebar_rollup();
      }, 6000);
      document.getElementById("pageinfosidebar").style.opacity = "100%";
    } else {
      document.getElementById("pageinfosidebar").style.opacity = "30%";
      document
        .getElementById("pageinfosidebar")
        .setAttribute("onmouseover", "this.style.opacity = '100%'");
      document
        .getElementById("pageinfosidebar")
        .setAttribute("onmouseout", "this.style.opacity = '30%'");
    }
  } else {
    console.info(
      `Not a post. Not displaying pageinfosidebar. Kind is '${cynthia.kind}'.`,
    );
    document.getElementById("pageinfosidebar").style.display = "none";
  }
}
