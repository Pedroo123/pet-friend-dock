import time
import json
import sys
import psutil

def get_pet_state():
    while True:
        # Porcentagem de uso da CPU (uma media dos nucleos de maneira geral)
        # Intervalo de 1.0 para nao ser uma consulta constante
        cpu_usage = psutil.cpu_percent(interval=1.0)

        # Estado do pet baseado no uso
        if cpu_usage < 15.0:
            state = "SLEEPING"
            speed = 0
        else:
            state = "RUNNING"
            # Divide e sempre arredonda
            speed = int(cpu_usage // 10) + 1

        #Objeto para a letirua do rust
        status = {
            "cpu": cpu_usage,
            "state": state,
            "speed": speed
        }

        print(json.dump(status))
        sys.stdout.flush()

if __name__ == "__main__":
    try:
        get_pet_state()
    except KeyboardInterrupt:
        sys.exit(0)

