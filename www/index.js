import * as wasm from "lift-wasm";

const canvas = document.getElementById("lift-canvas");
let isStopped = false;
let timeStopped = 0;


const toggledFloors = [];

let lastTimestamp;
const mainLoop = (timestamp) => {
    if (lastTimestamp === undefined) {
        lastTimestamp = timestamp;
    }
    const timeStep = (timestamp - lastTimestamp) / 1000.0;

    let lift;
    if (!isStopped || timeStopped > 1) {
        lift = wasm.step_simulation(timeStep);
        isStopped = lift.is_stopped;
        let floor = Math.round(lift.position);
        if (toggledFloors.includes(floor)) {
            toggleFloorIndicator(floor, false);
        }
        timeStopped = 0;
    } else {
        lift = wasm.last_simulation_result();
        timeStopped += timeStep;
    }

    const ctx = canvas.getContext("2d");
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    

    const liftHeight = canvas.height / 13;

    ctx.strokeStyle = "#AAA";
    for (let i = 1; i < 13; i++) {
        ctx.beginPath();
        ctx.moveTo(0, i * liftHeight);
        ctx.lineTo(canvas.width, i * liftHeight);
        ctx.stroke(); 
    }

    const liftOffset = liftHeight * 3;
    ctx.fillStyle = "#AdA425";
    ctx.fillRect(0, canvas.height - lift.position * liftHeight - liftOffset, canvas.width, liftHeight);

    toggledFloors.forEach(floor => {
        const timer = "floor-button-timer-" + floor;
        const remainingTime = wasm.time_to_floor(floor, 1.0);
        if (remainingTime !== undefined && remainingTime > 0) {
            document.getElementById(timer).innerText =  remainingTime.toFixed(1) + "s";
        }
    });

    lastTimestamp = timestamp;
    window.requestAnimationFrame(mainLoop);
};

const floorButtons = Array.from(document.getElementsByClassName("floor-button"));
const emergencyStopButton = document.getElementById("emergency-stop-button");

let emergencyStopStatus = false;
emergencyStopButton.addEventListener("click", () => {
    console.log("Emergency stop button pressed");
    emergencyStopStatus = !emergencyStopStatus;
    wasm.emergency_stop(emergencyStopStatus);
    if (emergencyStopStatus) {
        emergencyStopButton.classList.add("active-floor-button");
    } else {
        emergencyStopButton.classList.remove("active-floor-button");
    }
});

floorButtons.forEach(button => {
    button.addEventListener("click", () => {
        const floor = button.value;
        console.log("Floor button " + floor + " pressed")
        wasm.stop_lift_at_floor(button.value);
        toggleFloorIndicator(floor, true);
    });
    
});


const toggleFloorIndicator = (value, status) => {
    floorButtons
        .filter(button => button.value == value)
        .forEach(button => {
            const active = "active-floor-button";
            
            if (status) {
                button.classList.add(active);
                toggledFloors.push(parseInt(value));
            } else {
                button.classList.remove(active);
                while (toggledFloors.includes(value)) {
                    const index = toggledFloors.indexOf(value);
                    if (index > -1) {
                        toggledFloors.splice(index, 1);
                    }
                }
                const timer = "floor-button-timer-" + value;
                document.getElementById(timer).innerText = "";
            }
            
    });
}


window.requestAnimationFrame(mainLoop);