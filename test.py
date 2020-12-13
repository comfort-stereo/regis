from dis import dis

def add(left, right):
    return left + right

def function():
    this = 1
    that = 2
    return add(add(this, that), that)
    

print(dis(function))