from prefect import task, flow, get_run_logger
import time

@task(name="Say hello")
def say_hello():
    time.sleep(5)
    get_run_logger().info("Hello")

@task(name="Say a name")
def say_name(name: str): 
    time.sleep(5)
    get_run_logger().info(f"Hello {name}")

@task(name="Say goodbye")
def say_goodbye():
    time.sleep(2)
    get_run_logger().info("Goodbye")

@flow(name="Test Flow")
def say_hello_goodbye():
    say_hello()
    say_goodbye()


@flow(name="Test Flow with params")
def say_name_goodbye(name: str):
    say_name(name)
    say_goodbye()