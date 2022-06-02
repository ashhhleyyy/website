setTimeout(function() {
    if (new Date().getMonth() !== 5) return;
    const FLAGS = [
        ["#000000", "#a3a3a3", "#ffffff", "#800070"],
        ["#d60270", "#d60270", "#9b4f96", "#0038A8", "#0038A8"],
        ["#d62900", "#ff9b55", "#ffffff", "#d461a6", "#a50062"],
        ["#fff430", "#ffffff", "#9c59d1", "#000000"],
        ["#ff1b8d", "#ffda00", "#1bb3ff"],
        ["#ff0018", "#ffa52c", "#ffff41", "#008018", "#0000f9", "#86007d"],
        ["#55cdfc", "#f7a8b8", "#ffffff", "#f7a8b8", "#55cdfc"],
    ];
    const flag = FLAGS[Math.floor(Math.random() * FLAGS.length)];
    let s = ('%c' + ('●●'.repeat(Math.ceil(flag.length * (16 / 9)))) + '\n').repeat(flag.length).trimEnd();
    let c = flag.map(function (c) { return "background-color: " + c + "; color: " + c; });
    console.log(s, ...c);
    console.log('%cH%ca%cv%ce%c %cp%cr%ci%cd%ce%c %c<%c3','color: rgb(255.0,0.0,0.0);','color: rgb(255.0,117.6923076923077,0.0);','color: rgb(255.0,235.3846153846154,0.0);','color: rgb(156.92307692307693,255.0,0.0);','color: rgb(39.230769230769205,255.0,0.0);','color: rgb(0.0,255.0,78.46153846153852);','color: rgb(0.0,255.0,196.15384615384613);','color: rgb(0.0,196.15384615384613,255.0);','color: rgb(0.0,78.46153846153841,255.0);','color: rgb(39.23076923076909,0.0,255.0);','color: rgb(156.92307692307705,0.0,255.0);','color: rgb(255.0,0.0,235.38461538461544);','color: rgb(255.0,0.0,117.69230769230772);');
}, 1000);
