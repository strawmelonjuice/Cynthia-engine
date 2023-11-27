/*
Cynthia Client-side script.

This script is embedded into any page Cynthia serves, always just before the closing </html>.
*/


setInterval(() => {
  var elements = document.getElementsByClassName('unparsedtimestamp');
  for (let i = elements.length - 1; i >= 0; i--) {
    let timestamp = elements[i].innerHTML;
    console.log("Parsing timestamp.");
    const jstimestamp = timestamp * 1000;
    const dateObject = new Date(jstimestamp);
    const data = dateObject.toLocaleString();
    const date = data.substring(0, data.length - 3);
    elements[i].innerHTML = date;
    elements[i].classList.remove('unparsedtimestamp');
  }
}, 100);


function mobileorientation() {
  const csssays = getComputedStyle(document.body).getPropertyValue(
    "--screen-type-orientation"
  );
  if (csssays === "mobile") {
    return 1;
  }
  if (csssays === "landscape") {
    return 0;
  }
  console.error(
    "Could not determine 'mobilescreen()' from css value '" + csssays + "'."
  );
}
if (document.getElementById("cynthiapageinfoshowdummyelem") != null) {
  let pageinfosidebarelem = document.getElementById(
    "cynthiapageinfoshowdummyelem"
  );
  pageinfosidebarelem.setAttribute(
    "style",
    "opacity: 0.3; transition: all 2s ease-out 0s;"
  );
  pageinfosidebarelem.id = "pageinfosidebar";
  pageinfosidebarelem = document.getElementById("pageinfosidebar");
  let authorthumbnail = "";
  if (
    typeof pagemetainfo.author.thumbnail !== "undefined"
  ) {
    authorthumbnail = `<img style="width: 2.5em" src="${pagemetainfo.author.thumbnail}">`;
  }
  let dates = "";
  if (typeof pagemetainfo.dates !== "undefined") {
    if (pagemetainfo.dates.published == pagemetainfo.dates.altered || typeof pagemetainfo.dates.altered == "undefined") {
      dates = `<li>Posted: <span class="unparsedtimestamp">${(new Date((pagemetainfo.dates.published) * 1000).toLocaleString())}</span></li>`
    } else {
      dates = `
    <li>Posted: <span class="unparsedtimestamp">${(new Date((pagemetainfo.dates.published) * 1000).toLocaleString())}</span></li>
    <li>Edited: <span class="unparsedtimestamp">${(new Date((pagemetainfo.dates.altered) * 1000).toLocaleString())}</span></li>
    `
    }
  }
  pageinfosidebarelem.innerHTML = `
    <span class="not-on-mobile" style="position:absolute;right:0;top:0px;font-size: 3em; cursor: pointer; ">⇙</span>
    <p class="pageinfo-title">${pagemetainfo.title}</p>
    <ul>
      <li>Author: ${authorthumbnail} ${pagemetainfo.author.name}</li>
      ${dates}
      </ul>
    <p class="pageinfo-shortversion">${pagemetainfo.short}</p>
      `;
  function pageinfosidebar_rollup() {
    document.getElementById("pageinfosidebar").style.overflow = "hidden";
    document.getElementById("pageinfosidebar").style.width = "0px";
    document.getElementById("pageinfosidebar").style.maxHeight = "310px";
    document.getElementById("pageinfosidebartoggle").style.display = "";
    setTimeout(() => {
      document.getElementById("pageinfosidebartoggle").style.width = "40px";
    }, "1700");
    setTimeout(() => {
      document.getElementById("pageinfosidebartoggle").style.padding = "8px";
    }, "1800");
  }
  function pageinfosidebar_rollout() {
    document.getElementById("pageinfosidebar").style.overflow = "";
    document.getElementById("pageinfosidebar").style.opacity = "100%";
    document.getElementById("pageinfosidebartoggle").style.overflow = "hidden";
    document.getElementById("pageinfosidebartoggle").style.width = "0px";
    document.getElementById("pageinfosidebartoggle").style.padding = "0px";
    setTimeout(() => {
      document.getElementById("pageinfosidebar").style.width = "";
    }, "1800");
    setTimeout(() => {
      document.getElementById("pageinfosidebar").style.height = "";
    }, "1900");
  }

  if (mobileorientation() || window.innerHeight < 350) {
    setTimeout(() => {
      pageinfosidebar_rollup();
    }, "6000");
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
  $("#pageinfosidebar").on("click", function(e) {
    if (e.target == document.getElementById("dummyauthorthumbnail")) {
      GhostAuthorThumbnailExpand();
      return;
    }

    pageinfosidebar_rollup();
  });
}

