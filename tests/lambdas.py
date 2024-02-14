adder = lambda y: lambda z: y + z

add3 = adder(3)

print(add3(2))

x = [x*2 for x in range(10)]

x = list(map(lambda x: x / 2, x))
