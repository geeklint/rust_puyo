import { TwoPlayerGame } from "puyo_rust";

const game = TwoPlayerGame.new();
const renderLoop = () => {
    if (game.tick()) {
        requestAnimationFrame(renderLoop);
    }
};

requestAnimationFrame(renderLoop);

document.onkeydown = ((e) => {
    e = e || window.event;
    if (e.keyCode == 65){
        game.p1_left();
    } else if (e.keyCode == 68) {
        game.p1_right();
    } else if (e.keyCode == 87) {
        game.p1_up();
    } else if (e.keyCode == 83) {
        game.p1_down();
    } else if (e.keyCode == 32) {
        game.p1_rotate();
    } else if (e.keyCode == 37) {
        game.p2_left();
    } else if (e.keyCode == 39) {
        game.p2_right();
    } else if (e.keyCode == 38) {
        game.p2_up();
    } else if (e.keyCode == 40) {
        game.p2_down();
    } else if (e.keyCode == 13) {
        game.p2_rotate();
    } else if (e.keyCode == 89) {
        game.restart();
        requestAnimationFrame(renderLoop);
    }
});
